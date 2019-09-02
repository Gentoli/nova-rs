//! Loaders for user shaderpacks.
//!
//! There is currently a single entrypoint: [`load_nova_shaderpack`](shaderpack::load_nova_shaderpack).
//! Use this function to load a shaderpack from disk.
//!
//! TOOD(cwfitzgerald): Unify shaderpack entrypoints.

use crate::loading::{DirectoryFileTree, FileTree, LoadingError};
use failure::Error;
use failure::Fail;
use futures::task::SpawnExt;
use path_dsl::path;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

mod structs;
pub use structs::*;

/// Failure type for shaderpack loading.
#[derive(Fail, Debug)]
pub enum ShaderpackLoadingFailure {
    /// Path to the root of the shaderpack not found
    #[fail(display = "Path to shaderpack not found: {:?}", _0)]
    PathNotFound(PathBuf),

    /// If the shaderpack is a single file, it has an unknown extension
    #[fail(display = "Unsupported shaderpack extension {:?}", _0)]
    UnsupportedExtension(String),

    /// Required file not found inside shaderpack
    #[fail(display = "File {:?} not found in shaderpack.", _0)]
    MissingFile(OsString),

    /// Required directory not found inside shaderpack
    #[fail(display = "Directory {:?} not found in shaderpack.", _0)]
    MissingDirectory(OsString),

    /// Error while parsing shaderpack json
    #[fail(display = "Error while parsing json {:?}", _0)]
    JsonError(OsString, serde_json::Error),

    /// Shaderpack requires a certain path inside the shaderpack to be a
    /// directory, but hte shaderpack has it as a file.
    #[fail(display = "Directory member is a file not a directory {:?}", _0)]
    NotDirectory(OsString),

    /// Shaderpack requires a certain path inside the shaderpack to be a
    /// file, but hte shaderpack has it as a directory.
    #[fail(display = "Directory member is a directory not a file {:?}", _0)]
    NotFile(OsString),

    /// An unknown error occurred internally. This is generally a bug.
    #[fail(display = "Unknown internal error: {:?}", sub_error)]
    UnknownError {
        /// Actual error
        #[fail(cause)]
        sub_error: Error,
    },

    /// Error within the filesystem. Might be a bug.
    #[fail(display = "Unknown filesystem error: {:?}", sub_error)]
    FileSystemError {
        /// Actual error
        #[fail(cause)]
        sub_error: Error,
    },
}

/// Load a nova shaderpack from a file or folder.
///
/// File names are currently case sensitive.
///
/// # File Tree
///
/// - `passes.json`
/// - `resources.json`
/// - `materials`
///   - `*.mat`
///   - `*.pipeline`
/// - `shaders`
///   - `*.frag`
///   - `*.vert`
///
/// # File Formats
///
/// While the file tree must be the same, the shaderpacks can either come as an unpacked folder
/// or as one of the following single-file formats:
/// - None
///
/// Future Supported Formats:
/// - BZIP2/Deflate/Uncompressed `.zip`
/// - TAR (maybe)
/// - LZMA2 `.7z` (maybe)
///
/// # Arguments
///
/// - `executor` - Executor to run sub-tasks on
/// - `path` - Path to the root of the shaderpack, or the file the shaderpack is contained in.
pub async fn load_nova_shaderpack<E>(executor: E, path: PathBuf) -> Result<ShaderpackData, ShaderpackLoadingFailure>
where
    E: SpawnExt + Clone + 'static,
{
    // This function is a wrapper which properly dispatches to various sub functions

    // This should actually really be a if let chain, but that's not in the language yet
    match (path.exists(), path.is_dir(), path.extension().and_then(|s| s.to_str())) {
        // Directory
        (true, true, _) => {
            // Get the file tree
            let file_tree_res: Result<DirectoryFileTree, _> = DirectoryFileTree::from_path(&path).await;

            // Map error from the LoadingError type to the ShaderpackLoading Failure type
            let file_tree = file_tree_res.map_err(|err| match err {
                LoadingError::ResourceNotFound => ShaderpackLoadingFailure::PathNotFound(path),
                LoadingError::FileSystemError { sub_error: e } => {
                    ShaderpackLoadingFailure::FileSystemError { sub_error: e }
                }
                e => ShaderpackLoadingFailure::UnknownError { sub_error: e.into() },
            })?;

            // Actually load the file path
            load_nova_shaderpack_impl(executor, file_tree).await
        }
        // Zip File
        (true, false, Some("zip")) => unimplemented!(),
        // File with unknown extant
        (true, false, Some(ext)) => Err(ShaderpackLoadingFailure::UnsupportedExtension(ext.to_owned())),
        // File with no extant
        (true, false, None) => Err(ShaderpackLoadingFailure::UnsupportedExtension("<blank>".into())),
        // Path doesn't exist
        (false, _, _) => Err(ShaderpackLoadingFailure::PathNotFound(path)),
    }
}

/// Properly handles launching an async task on a executor and
/// gives back a RemoteHandle.
///
/// Will get replaced with a proper async macro
macro_rules! shaderpack_load_invoke {
    ( into: $typ:ty, $exec:expr, $($args:expr),* ) => {
        $exec.spawn_with_handle(load_json::<$typ, T>($($args),*)).unwrap()
    };
}

// Will get moved to async helpers
macro_rules! await_result_vector {
    ($vec:expr ) => {{
        let mut vec = Vec::new();
        vec.reserve($vec.len());
        for f in $vec {
            vec.push(f.await?);
        }
        vec
    }};
}

async fn load_nova_shaderpack_impl<E, T>(mut executor: E, tree: T) -> Result<ShaderpackData, ShaderpackLoadingFailure>
where
    E: SpawnExt + Clone + 'static,
    T: FileTree + Send + Clone + 'static,
{
    // To maximize parallelism in an highly async function, you need to dispatch new tasks as soon as you can,
    // and wait on their results as late as you can. This way you give each async job as much time as possible to
    // finish, so when you need the result, it is hopefully already ready.
    //
    // This can make the code a bit more convoluted, but nothing some good commenting can solve.

    // //////////// //
    // Job Creation //
    // //////////// //

    // Dispatch the job to load the "passes.json" file
    let passes_fut = shaderpack_load_invoke!(
        into: Vec<RenderPassCreationInfo>,
        executor,
        tree.clone(),
        "passes.json".into()
    );

    // Dispatch the job to load the "resources.json" file
    let resources_fut = shaderpack_load_invoke!(
        into: ShaderpackResourceData,
        executor,
        tree.clone(),
        "resources.json".into()
    );

    // While those operations are going, get a list of files in the materials folder. Because
    // of how the loading system work, the file tree is already populated, so this is a fully
    // synchronous memory operation.
    let materials_folder = enumerate_folder(&tree, "materials")?;

    // We have many files to load, create vectors.
    let mut materials_futs = Vec::new();
    let mut pipelines_futs = Vec::new();

    // Iterate through the materials directory to find the useful files in the files with the needed extant
    for path in materials_folder {
        let full_path = path!("materials" | &path).into();
        let ext = path.extension().and_then(|s| s.to_str());
        // Match on the extension
        match ext {
            Some("mat") => {
                let fut = shaderpack_load_invoke!(into: MaterialData, executor, tree.clone(), full_path);
                materials_futs.push(fut)
            }
            Some("pipeline") => {
                let fut = shaderpack_load_invoke!(into: PipelineCreationInfo, executor, tree.clone(), full_path);
                pipelines_futs.push(fut)
            }
            // We give no fucks about any other files
            _ => {}
        }
    }

    // We do the same for the shaders folder, but just blanket loading everything
    let shaders_folder: HashSet<PathBuf> = enumerate_folder(&tree, "shaders")?
        .into_iter()
        .map(|path| path!("shaders" | path).into())
        .collect();

    let shader_futs: Vec<_> = shaders_folder.iter().map(|p| tree.read_text(p)).collect();
    // Generate a mapping from path to an index for all shaders
    // This allows us to load each file only once.
    let shader_mapping: HashMap<&PathBuf, u32> =
        shaders_folder.iter().enumerate().map(|(i, p)| (p, i as u32)).collect();

    // ////////////// //
    // Job Resolution //
    // ////////////// //

    // Pull all materials files first as we can do something with them
    let mut materials = await_result_vector!(materials_futs);
    // We have all the data we need to do the materials postprocess pass
    set_material_pass_material_name(&mut materials);

    // Pull all pipelines as we also can do stuff with them immediately
    let mut pipelines = await_result_vector!(pipelines_futs);
    pipeline_postprocess(&mut pipelines, shader_mapping);

    let shaders = ShaderSet::Sources({
        let mut vec = Vec::with_capacity(shader_futs.len());

        // Futures are async, but are the actual handles themselves are in the same order
        // as the filenames, so can be safely zip together
        for (fut, filename) in shader_futs.into_iter().zip(shaders_folder.into_iter()) {
            // Await the future and translate the error
            let source = fut.await.map_err(|err| match err {
                LoadingError::NotFile => ShaderpackLoadingFailure::NotFile(filename.clone().into_os_string()),
                LoadingError::FileSystemError { sub_error } => ShaderpackLoadingFailure::FileSystemError { sub_error },
                LoadingError::PathNotFound => ShaderpackLoadingFailure::MissingFile(filename.clone().into_os_string()),
                e => ShaderpackLoadingFailure::UnknownError { sub_error: e.into() },
            })?;
            vec.push(LoadedShader { filename, source });
        }
        vec
    });

    // These weren't actually needed until right now, so there's no point in
    // awaiting their futures until they are needed.

    // Get the "passes.json" file
    let passes = passes_fut.await?;

    // Get the "resources.json" file
    let resources = resources_fut.await?;

    Ok(ShaderpackData {
        passes,
        resources,
        materials,
        pipelines,
        shaders,
    })
}

/// Each [`MaterialPass`] needs to have it's material name be
/// set from the parent material. This is hard to do in serde, so
/// serde ignores it and it is set in this pass.
fn set_material_pass_material_name(materials: &mut [MaterialData]) {
    for mat in materials {
        for pass in &mut mat.passes {
            pass.material_name = mat.name.clone();
        }
    }
}

/// During loading, a ShaderSource is a path to a shader file. These have been
/// loaded into an array of shader sources. Using the mapping of path to index we generated before,
/// we not replace the path with a index.
fn pipeline_postprocess(pipelines: &mut [PipelineCreationInfo], shader_mapping: HashMap<&PathBuf, u32>) {
    // A helpful closure that processes a single shader. Needs to be a closure
    // because it captures the surrounding arguments.
    let process_shader = |shader: &mut ShaderSource| {
        if let ShaderSource::Path(name) = shader {
            *shader = match shader_mapping.get(name) {
                Some(index) => ShaderSource::Loaded(*index),
                None => ShaderSource::Invalid,
            }
        } else {
            panic!("Invalid ShaderSource state. {:?}", shader);
        }
    };

    // Forwarding wrapper that unwraps an optional shader.
    let process_shader_option = |shader_option: &mut Option<ShaderSource>| {
        if let Some(shader) = shader_option {
            process_shader(shader)
        }
        // Does nothing if it doesn't exist
    };

    for pipeline in pipelines {
        process_shader(&mut pipeline.vertex_shader);
        process_shader_option(&mut pipeline.tessellation_control_shader);
        process_shader_option(&mut pipeline.tessellation_evaluation_shader);
        process_shader_option(&mut pipeline.geometry_shader);
        process_shader_option(&mut pipeline.fragment_shader);
    }
}

/// Helper function that enumerates the contents of a folder. Is a wrapper for [`FileTree::read_dir`]
/// that also properly changes the errors to the proper format
fn enumerate_folder<T, P>(tree: &T, path: P) -> Result<HashSet<PathBuf>, ShaderpackLoadingFailure>
where
    T: FileTree,
    P: AsRef<Path> + Into<OsString>,
{
    tree.read_dir(path.as_ref()).map_err(|err| match err {
        LoadingError::PathNotFound => ShaderpackLoadingFailure::MissingDirectory(path.into()),
        LoadingError::FileSystemError { sub_error: e } => ShaderpackLoadingFailure::FileSystemError { sub_error: e },
        e => ShaderpackLoadingFailure::UnknownError { sub_error: e.into() },
    })
}

/// Helper function that loads an json file from the file tree, then uses serde to deserialize it into
/// R. It then properly deals with that error. The type to deserialize into is through return type deduction,
/// so to invoke by an executor macro, you need to use superfish.
async fn load_json<R, T>(tree: T, path: PathBuf) -> Result<R, ShaderpackLoadingFailure>
where
    R: serde::de::DeserializeOwned + Send,
    T: FileTree + Send,
{
    // Load the json file, we need the result immediately before we can proceed, so await it.
    // This isn't launched on the executor because it is not an async function itself, it's
    // a piece of async io.
    let rp_file_result: Result<Vec<u8>, _> = tree.read(path.as_ref()).await;

    // Convert the errors
    let rp_file = rp_file_result.map_err(|err| match err {
        LoadingError::NotFile => ShaderpackLoadingFailure::NotFile(path.clone().into_os_string()),
        LoadingError::FileSystemError { sub_error } => ShaderpackLoadingFailure::FileSystemError { sub_error },
        LoadingError::PathNotFound => ShaderpackLoadingFailure::MissingFile(path.clone().into_os_string()),
        e => ShaderpackLoadingFailure::UnknownError { sub_error: e.into() },
    })?;

    // Deserialize the json
    let parsed: Result<R, _> = serde_json::from_slice(&rp_file);
    // Map the json error
    parsed.map_err(|err| ShaderpackLoadingFailure::JsonError(path.into_os_string(), err))
}
