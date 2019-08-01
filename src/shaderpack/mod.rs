//! Shaderpack loaders.

use crate::loading::{DirectoryFileTree, FileTree, LoadingError};
use failure::Error;
use failure::Fail;
use futures::StreamExt;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

mod structs;
use std::collections::HashSet;
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

pub async fn load_nova_shaderpack(path: PathBuf) -> Result<ShaderpackData, ShaderpackLoadingFailure> {
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
            load_nova_shaderpack_impl(&file_tree).await
        }
        (true, false, Some("zip")) => unimplemented!(),
        (true, false, Some(ext)) => Err(ShaderpackLoadingFailure::UnsupportedExtension(ext.to_owned())),
        (true, false, None) => Err(ShaderpackLoadingFailure::UnsupportedExtension("<blank>".into())),
        (false, _, _) => Err(ShaderpackLoadingFailure::PathNotFound(path)),
    }
}

async fn load_nova_shaderpack_impl<'a, T: FileTree<'a>>(
    tree: &'a T,
) -> Result<ShaderpackData, ShaderpackLoadingFailure> {
    let passes: Vec<RenderPassCreationInfo> = load_json(tree, &"/passes.json").await?;
    let resources: ShaderpackResourceData = load_json(tree, &"/resources.json").await?;
    let materials_folder = enumerate_folder(tree, &"/materials")?;
    let mut materials: Vec<MaterialData> = Vec::new();
    let mut pipelines: Vec<PipelineCreationInfo> = Vec::new();
    for path in materials_folder {
        let full_path = format!("/materials/{}", path.to_string_lossy());
        let ext = path.extension().and_then(|s| s.to_str());
        match ext {
            Some("mat") => materials.push(load_json(tree, full_path).await?),
            Some("pipeline") => pipelines.push(load_json(tree, full_path).await?),
            _ => {}
        }
    }

    Ok(ShaderpackData {
        passes,
        resources,
        materials,
        pipelines,
    })
}

fn enumerate_folder<'a, T, P>(tree: &'a T, path: P) -> Result<HashSet<&'a Path>, ShaderpackLoadingFailure>
where
    T: FileTree<'a>,
    P: AsRef<Path> + Into<OsString>,
{
    tree.read_dir(path.as_ref())
        .map_err(|err| match err {
            LoadingError::PathNotFound => ShaderpackLoadingFailure::MissingDirectory(path.into()),
            LoadingError::FileSystemError { sub_error: e } => {
                ShaderpackLoadingFailure::FileSystemError { sub_error: e }
            }
            e => ShaderpackLoadingFailure::UnknownError { sub_error: e.into() },
        })
        .map(|iter| iter.collect())
}

async fn load_json<'a, R, T, P>(tree: &'a T, path: P) -> Result<R, ShaderpackLoadingFailure>
where
    R: serde::de::DeserializeOwned,
    T: FileTree<'a>,
    P: AsRef<Path>,
{
    let rp_file_result: Result<Vec<u8>, _> = tree.read(path.as_ref()).await;
    let rp_file = rp_file_result.map_err(|err| match err {
        LoadingError::NotFile => ShaderpackLoadingFailure::NotFile(path.as_ref().into()),
        LoadingError::FileSystemError { sub_error } => ShaderpackLoadingFailure::FileSystemError { sub_error },
        LoadingError::PathNotFound => ShaderpackLoadingFailure::MissingFile(path.as_ref().into()),
        e => ShaderpackLoadingFailure::UnknownError { sub_error: e.into() },
    })?;
    let parsed: Result<R, _> = serde_json::from_slice(&rp_file);
    parsed.map_err(|err| ShaderpackLoadingFailure::JsonError(path.as_ref().into(), err))
}
