//! Loaders for user shaderpacks.
//!
//! There is currently a single entrypoint: [`load_nova_shaderpack`].
//! Use this function to load a shaderpack from disk.
//!
//! TOOD(cwfitzgerald): Unify shaderpack entrypoints.

use crate::loading::{DirectoryFileTree, FileTree, LoadingError};
use failure::Error;
use failure::Fail;
use futures::task::SpawnExt;
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
    match (path.exists(), path.is_dir(), path.extension().and_then(|s| s.to_str())) {
        (true, true, _) => {
            let file_tree_res: Result<DirectoryFileTree, _> = DirectoryFileTree::from_path(&path).await;
            let file_tree = file_tree_res.map_err(|err| match err {
                LoadingError::ResourceNotFound => ShaderpackLoadingFailure::PathNotFound(path),
                LoadingError::FileSystemError { sub_error: e } => {
                    ShaderpackLoadingFailure::FileSystemError { sub_error: e }
                }
                e => ShaderpackLoadingFailure::UnknownError { sub_error: e.into() },
            })?;
            load_nova_shaderpack_impl(executor, file_tree).await
        }
        (true, false, Some("zip")) => unimplemented!(),
        (true, false, Some(ext)) => Err(ShaderpackLoadingFailure::UnsupportedExtension(ext.to_owned())),
        (true, false, None) => Err(ShaderpackLoadingFailure::UnsupportedExtension("<blank>".into())),
        (false, _, _) => Err(ShaderpackLoadingFailure::PathNotFound(path)),
    }
}

macro_rules! shaderpack_load_invoke {
    ( into: $typ:ty, $exec:expr, $($args:expr),* ) => {
        $exec.spawn_with_handle(load_json::<$typ, T>($($args),*)).unwrap()
    };
}

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

// TODO(cwfitzgerald): This code is complicated af, comment it up
async fn load_nova_shaderpack_impl<E, T>(mut executor: E, tree: T) -> Result<ShaderpackData, ShaderpackLoadingFailure>
where
    E: SpawnExt + Clone + 'static,
    T: FileTree + Send + Clone + 'static,
{
    // //////////// //
    // Job Creation //
    // //////////// //

    let passes_fut = shaderpack_load_invoke!(
        into: Vec<RenderPassCreationInfo>,
        executor,
        tree.clone(),
        "passes.json".into()
    );

    let resources_fut = shaderpack_load_invoke!(
        into: ShaderpackResourceData,
        executor,
        tree.clone(),
        "resources.json".into()
    );

    let materials_folder = enumerate_folder(&tree, "materials")?;

    let mut materials_futs = Vec::new();
    let mut pipelines_futs = Vec::new();

    for path in materials_folder {
        let full_path = {
            let mut p = PathBuf::new();
            p.push("materials");
            p.push(&path);
            p
        };
        let ext = path.extension().and_then(|s| s.to_str());
        match ext {
            Some("mat") => {
                let fut = shaderpack_load_invoke!(into: MaterialData, executor, tree.clone(), full_path);
                materials_futs.push(fut)
            }
            Some("pipeline") => {
                let fut = shaderpack_load_invoke!(into: PipelineCreationInfo, executor, tree.clone(), full_path);
                pipelines_futs.push(fut)
            }
            _ => {}
        }
    }

    let shaders_folder: HashSet<PathBuf> = enumerate_folder(&tree, "shaders")?
        .into_iter()
        .map(|path| {
            let mut p = PathBuf::new();
            p.push("shaders");
            p.push(path);
            p
        })
        .collect();

    let shader_futs: Vec<_> = shaders_folder.iter().map(|p| tree.read_text(p)).collect();
    let shader_mapping: HashMap<&PathBuf, u32> =
        shaders_folder.iter().enumerate().map(|(i, p)| (p, i as u32)).collect();

    // ////////////// //
    // Job Resolution //
    // ////////////// //

    let passes = passes_fut.await?;
    let resources = resources_fut.await?;
    let mut materials = await_result_vector!(materials_futs);
    material_postprocess(&mut materials);
    let mut pipelines = await_result_vector!(pipelines_futs);
    pipeline_postprocess(&mut pipelines, shader_mapping);
    let shaders = ShaderSet::Sources({
        let mut vec = Vec::new();
        vec.reserve(shader_futs.len());
        for (fut, filename) in shader_futs.into_iter().zip(shaders_folder.into_iter()) {
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

    Ok(ShaderpackData {
        passes,
        resources,
        materials,
        pipelines,
        shaders,
    })
}

fn material_postprocess(materials: &mut [MaterialData]) {
    for mat in materials {
        for pass in &mut mat.passes {
            pass.material_name = mat.name.clone();
        }
    }
}

fn pipeline_postprocess(pipelines: &mut [PipelineCreationInfo], shader_mapping: HashMap<&PathBuf, u32>) {
    let process_shader = |shader: &mut ShaderSource| {
        if let ShaderSource::Path(name) = shader {
            *shader = match shader_mapping.get(name) {
                Some(index) => ShaderSource::Loaded(*index),
                None => ShaderSource::Invalid,
            }
        }
    };

    let process_shader_option = |shader_option: &mut Option<ShaderSource>| {
        if let Some(shader) = shader_option {
            process_shader(shader)
        }
    };

    for pipeline in pipelines {
        process_shader(&mut pipeline.vertex_shader);
        process_shader_option(&mut pipeline.tessellation_control_shader);
        process_shader_option(&mut pipeline.tessellation_evaluation_shader);
        process_shader_option(&mut pipeline.geometry_shader);
        process_shader_option(&mut pipeline.fragment_shader);
    }
}

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

async fn load_json<R, T>(tree: T, path: PathBuf) -> Result<R, ShaderpackLoadingFailure>
where
    R: serde::de::DeserializeOwned + Send,
    T: FileTree + Send,
{
    let rp_file_result: Result<Vec<u8>, _> = tree.read(path.as_ref()).await;
    let rp_file = rp_file_result.map_err(|err| match err {
        LoadingError::NotFile => ShaderpackLoadingFailure::NotFile(path.clone().into_os_string()),
        LoadingError::FileSystemError { sub_error } => ShaderpackLoadingFailure::FileSystemError { sub_error },
        LoadingError::PathNotFound => ShaderpackLoadingFailure::MissingFile(path.clone().into_os_string()),
        e => ShaderpackLoadingFailure::UnknownError { sub_error: e.into() },
    })?;
    let parsed: Result<R, _> = serde_json::from_slice(&rp_file);
    parsed.map_err(|err| ShaderpackLoadingFailure::JsonError(path.into_os_string(), err))
}
