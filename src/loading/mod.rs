//! Nova's file loading code.
//!
//! Implements a resource pack loader, which may be used for loading Optifine shaderpacks, Minecraft: Java Edition
//! resourcepacks, Bedrock engine resourcepacks, and Nova shaderpacks. It will accomplish this by not knowing about any
//! of those and will instead only take in file paths and will return either streams of bytes or strings. The resource
//! pack loader will also be able to read resource packs in either filesystem folders or a zip folder. It should be
//! constructed in a way that will allow support for other zip formats.

use failure::{Error, Fail};
use futures::Future;
use std::path::{Path, PathBuf};

mod dir;

pub use dir::*;
use std::collections::HashSet;

/// View over an abstract tree of directories and files.
///
/// Used to abstract over the actual backend being used to allow a wider variety of formats. Such examples could be a
/// filesystem folder, a zip file, or a network store.
///
/// All paths passes in are relative to the `FileTree` root, and paths which go outside this `FileTree` are not allowed.
/// The one exception is the [`from_path`](FileTree::from_path) method which accepts a path to the `FileTree` root.
pub trait FileTree: Sized {
    // /// Iterator type to iterate over the members of a directory.
    // // This will not compile without GAT, and that's a while out.
    // // Use a Vec<PathBuf> instead
    // type DirIter<'a>: Iterator<Item = &'a Path>;

    /// Create a file tree from the path provided.
    ///
    /// May be expensive depending on the target you are opening.
    fn from_path(path: &Path) -> Self::FromPathResult;
    /// Associated return type for [`FileTree::from_path`].
    ///
    /// Stopgap until async fn in traits happens.
    type FromPathResult: Future<Output = Result<Self, LoadingError>> + Send;

    /// Checks is file path exists within the current file tree.
    fn exists(&self, path: &Path) -> bool;

    /// Checks if the path points to a file.
    ///
    /// File Exists -> `Ok(true)`
    /// Exists but isn't file -> `Ok(false)`
    /// Path doesn't exist -> [`LoadingError::PathNotFound`]
    fn is_file(&self, path: &Path) -> Result<bool, LoadingError>;

    /// Checks if the path points to a directory.
    ///
    /// Directory Exists -> `Ok(true)`
    /// Exists but isn't directory -> `Ok(false)`
    /// Path doesn't exist -> [`LoadingError::PathNotFound`]
    fn is_dir(&self, path: &Path) -> Result<bool, LoadingError>;

    /// Returns an ~~iterator~~ [`HashSet`] over all paths in the specified directory.
    ///
    /// Fails if the directory doesn't exist, or is unreadable.
    fn read_dir(&self, path: &Path) -> Result<HashSet<PathBuf>, LoadingError>;

    /// Reads a file into a vector of u8.
    ///
    /// Fails if file doesn't exist or isn't readable.
    fn read(&self, path: &Path) -> Self::ReadResult;
    /// Associated return type for [`FileTree::read`].
    ///
    /// Stopgap until async fn in traits happens.
    type ReadResult: Future<Output = Result<Vec<u8>, LoadingError>> + Send;

    /// Reads a file as little endian into an array of u32.
    ///
    /// Fails if file doesn't exist or isn't readable.
    fn read_u32(&self, path: &Path) -> Self::ReadU32Result;
    /// Associated return type for [`FileTree::read_u32`].
    ///
    /// Stopgap until async fn in traits happens.
    type ReadU32Result: Future<Output = Result<Vec<u32>, LoadingError>> + Send;

    /// Reads a file as little endian into an array of u32.
    ///
    /// Fails if file doesn't exist or isn't readable.
    fn read_text(&self, path: &Path) -> Self::ReadTextResult;
    /// Associated return type for [`FileTree::read_u32`].
    ///
    /// Stopgap until async fn in traits happens.
    type ReadTextResult: Future<Output = Result<String, LoadingError>> + Send;
}

/// Error when trying to load a resource.
#[derive(Debug, Fail)]
pub enum LoadingError {
    /// Given path to resource does not exist.
    #[fail(display = "Given path to resource does not exist.")]
    ResourceNotFound,

    /// Path does not exist in resource.
    #[fail(display = "Path does not exist in resource.")]
    PathNotFound,

    /// Expected a directory, but found a file.
    #[fail(display = "Expected directory.")]
    NotDirectory,

    /// Expected a file, but found a directory.
    #[fail(display = "Expected file.")]
    NotFile,

    /// Error within the filesystem.
    #[fail(display = "Error inside filesystem.")]
    FileSystemError {
        /// Actual error
        #[fail(cause)]
        sub_error: Error,
    },
}
