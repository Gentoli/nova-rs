//! Shaderpack loaders.

use crate::loading::{DirectoryFileTree, FileTree, LoadingError};
use failure::Error;
use failure::Fail;
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
    JsonError(serde_json::Error),

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

async fn load_nova_shaderpack_impl<'a, T: FileTree<'a>>(tree: &T) -> Result<ShaderpackData, ShaderpackLoadingFailure> {
    let renderpasses = load_nova_renderpasses(tree).await;
    unimplemented!()
}

async fn load_nova_renderpasses<'a, T: FileTree<'a>>(
    tree: &T,
) -> Result<RenderPassCreationInfo, ShaderpackLoadingFailure> {
    let path = Path::new("/passes.json");
    let rp_file_result: Result<Vec<u8>, _> = tree.read(Path::new("/passes.json")).await;
    let rp_file = rp_file_result.map_err(|err| match err {
        LoadingError::NotFile => ShaderpackLoadingFailure::NotFile(path.into()),
        LoadingError::FileSystemError { sub_error } => ShaderpackLoadingFailure::FileSystemError { sub_error },
        LoadingError::PathNotFound => ShaderpackLoadingFailure::MissingFile(path.into()),
        e => ShaderpackLoadingFailure::UnknownError { sub_error: e.into() },
    })?;
    let parsed: Result<RenderPassCreationInfo, _> = serde_json::from_slice(&rp_file);
    parsed.map_err(|err| ShaderpackLoadingFailure::JsonError(err))
}
