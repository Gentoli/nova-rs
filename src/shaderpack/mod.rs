//! Shaderpack loaders.

use crate::loading::{DirectoryFileTree, FileTree, LoadingError};
use failure::Error;
use failure::Fail;
use futures::future::{join_all, RemoteHandle};
use futures::task::SpawnExt;
use futures::StreamExt;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

mod structs;
pub use structs::*;

#[derive(Fail, Debug)]
pub enum ShaderpackLoadingFailure {
    #[fail(display = "Path to shaderpack not found: {:?}", _0)]
    PathNotFound(PathBuf),

    #[fail(display = "Unsupported shaderpack extension {:?}", _0)]
    UnsupportedExtension(String),

    #[fail(display = "File {:?} not found in shaderpack.", _0)]
    MissingFile(OsString),

    #[fail(display = "File {:?} not found in shaderpack.", _0)]
    MissingDirectory(OsString),

    #[fail(display = "Error while parsing json {:?}", _0)]
    JsonError(OsString, serde_json::Error),

    #[fail(display = "Directory member is a file not a directory {:?}", _0)]
    NotDirectory(OsString),

    #[fail(display = "Directory member is a directory not a file {:?}", _0)]
    NotFile(OsString),

    #[fail(display = "Unknown internal error: {:?}", sub_error)]
    UnknownError {
        /// Actual error
        #[fail(cause)]
        sub_error: Error,
    },

    /// Error within the filesystem.
    #[fail(display = "Unknown filesystem error: {:?}", sub_error)]
    FileSystemError {
        /// Actual error
        #[fail(cause)]
        sub_error: Error,
    },
}

pub async fn load_nova_shaderpack<E>(executor: E, path: PathBuf) -> Result<ShaderpackData, ShaderpackLoadingFailure>
where
    E: SpawnExt + Clone + 'static,
{
    match (
        path.exists(),
        path.is_dir(),
        path.extension().iter().flat_map(|s| s.to_str()).next(),
    ) {
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
    ($vec:expr ) => {
        {
            let mut vec = Vec::new();
            vec.reserve($vec.len());
            for f in $vec {
                vec.push(f.await?);
            }
            vec
        }
    };
    ( $($vec:expr),+ ) => {
        ($(await_result_vector!($vec)),*)
    };
}

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
    // ////////////// //
    // Job Resolution //
    // ////////////// //

    let passes = passes_fut.await?;
    let resources = resources_fut.await?;
    let mut materials = await_result_vector!(materials_futs);
    material_postprocess(&mut materials);
    let pipelines = await_result_vector!(pipelines_futs);

    Ok(ShaderpackData {
        passes,
        resources,
        materials,
        pipelines,
    })
}

fn material_postprocess(materials: &mut [MaterialData]) {
    for mat in materials {
        for pass in &mut mat.passes {
            pass.material_name = mat.name.clone();
        }
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
