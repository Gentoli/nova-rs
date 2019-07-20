//! Nova's file loading code
//!
//! Implements a resource pack loader, which may be used for loading Optifine shaderpacks, Minecraft: Java Edition
//! resourcepacks, Bedrock engine resourcepacks, and Nova shaderpacks. It will accomplish this by not knowing about any
//! of those and will instead only take in file paths and will return either streams of bytes or strings. The resource
//! pack loader will also be able to read resource packs in either filesystem folders or a zip folder. It should be
//! constructed in a way that will allow support for other zip formats

use failure::{Error, Fail};
use futures::Future;
use std::path::Path;

mod dir;

pub use dir::*;

pub trait FileTree<'a> {
    type CreateResult: FileTree<'a>;
    type DirIter: Iterator<Item = &'a Path>;

    /// Create a file tree from the path provided.
    /// May be expensive depending on the target you are opening.
    fn from_path(path: &Path) -> Box<dyn Future<Output = Result<Self::CreateResult, LoadingError>>>;

    /// Checks is file path exists within the current file tree.
    fn exists(&self, path: &Path) -> bool;

    /// Checks if the path points to a file.
    ///
    /// File Exists -> `Some(true)`
    /// Exists but isn't file -> `Some(false)`
    /// Path doesn't exist -> `None`
    fn is_file(&self, path: &Path) -> Option<bool>;

    /// Checks if the path points to a directory.
    ///
    /// Directory Exists -> `Some(true)`
    /// Exists but isn't directory -> `Some(false)`
    /// Path doesn't exist -> `None`
    fn is_dir(&self, path: &Path) -> Option<bool>;

    /// Returns an iterator over all paths in the specified directory.
    ///
    /// Fails if the directory doesn't exist, or is unreadable.
    fn read_dir(&self, path: &Path) -> Result<Self::DirIter, Error>;

    /// Reads a file into a vector of u8.
    ///
    /// Fails if file doesn't exist or isn't readable.
    fn read(&self, path: &Path) -> Box<dyn Future<Output = Result<Vec<u8>, Error>>>;

    /// Reads a file as little endian into an array of u32.
    ///
    /// Fails if file doesn't exist or isn't readable.
    fn read_u32(&self, path: &Path) -> Box<dyn Future<Output = Result<Vec<u32>, Error>>>;

    /// Reads a file as little endian into an array of u32.
    ///
    /// Fails if file doesn't exist or isn't readable.
    fn read_text(&self, path: &Path) -> Box<dyn Future<Output = Result<String, Error>>>;
}

#[derive(Debug, Fail)]
pub enum LoadingError {
    #[fail(display = "Path doesn't exist.")]
    PathNotFound,
    #[fail(display = "Expected directory.")]
    NotDirectory,
    #[fail(display = "Expected file.")]
    NotFile,
}
