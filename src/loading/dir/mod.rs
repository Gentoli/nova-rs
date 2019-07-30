use crate::core::reactor::SingleThreadReactor;
use crate::fs::dir::{DirectoryEntry, DirectoryTree};
use crate::loading::{FileTree, LoadingError};
use futures::Future;
use matches::matches;
use std::io;
use std::path::Path;
use std::sync::Arc;

mod iter;
mod reactor;

pub use iter::*;
use reactor::*;
use std::pin::Pin;

/// File tree structure representing a filesystem directory.
///
/// It is a thin [`Arc`](std::sync::Arc) wrapper around the actual
/// internal [`DirectoryFileTreeData`](DirectoryFileTreeData) structure.
pub struct DirectoryFileTree(Arc<DirectoryFileTreeData>);

/// Actual data-holding structure for a fs directory tree.
struct DirectoryFileTreeData {
    cache: DirectoryTree,
    reactor: SingleThreadReactor<FileSystemOp, FileSystemOpResult>,
}

impl DirectoryFileTree {
    fn get_node_at_location(&self, path: &Path) -> Option<&DirectoryEntry> {
        self.0.cache.entry.get(path)
    }
}

impl<'a> FileTree<'a> for DirectoryFileTree {
    type CreateResult = Self;
    type DirIter = DirectoryIterator<'a>;

    fn from_path(path: &Path) -> Pin<Box<dyn Future<Output = Result<Self::CreateResult, LoadingError>>>> {
        let path = path.to_path_buf();
        Pin::from(Box::new(async move {
            if !path.exists() {
                return Err(LoadingError::ResourceNotFound);
            }
            if !path.is_dir() {
                return Err(LoadingError::NotDirectory);
            }

            let reactor = SingleThreadReactor::from_action(file_system_reactor_core);

            let future = reactor.send_async(FileSystemOp::RecursiveEnumerate(path));

            match future.await {
                FileSystemOpResult::RecursiveEnumerate(cache) => {
                    Ok(Self(Arc::new(DirectoryFileTreeData { cache, reactor })))
                }
                FileSystemOpResult::Error(err) => Err(LoadingError::FileSystemError { sub_error: err.into() }),
                _ => panic!("Incorrect directory action response received"),
            }
        }))
    }

    fn exists(&self, path: &Path) -> bool {
        self.get_node_at_location(path).is_some()
    }

    fn is_file(&self, path: &Path) -> Result<bool, LoadingError> {
        self.get_node_at_location(path)
            .map(|v| matches!(v, DirectoryEntry::File))
            .ok_or(LoadingError::PathNotFound)
    }

    fn is_dir(&self, path: &Path) -> Result<bool, LoadingError> {
        self.get_node_at_location(path)
            .map(|v| matches!(v, DirectoryEntry::Directory { .. }))
            .ok_or(LoadingError::PathNotFound)
    }

    fn read_dir(&'a self, path: &Path) -> Result<Self::DirIter, LoadingError> {
        match self.get_node_at_location(path) {
            Some(DirectoryEntry::File) => Err(LoadingError::NotDirectory),
            Some(DirectoryEntry::Directory { entries: map }) => Ok(map.keys().into()),
            None => Err(LoadingError::PathNotFound),
        }
    }

    fn read(&self, path: &Path) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, LoadingError>>>> {
        let path = path.to_path_buf();
        let data = self.0.clone();
        Pin::from(Box::new(async move {
            let real_path = {
                let mut p = data.cache.root.clone();
                p.push(path);
                p
            };
            let future = data.reactor.send_async(FileSystemOp::FileRead(real_path));

            match future.await {
                FileSystemOpResult::Error(error) => match error.kind() {
                    io::ErrorKind::NotFound => Err(LoadingError::PathNotFound),
                    _ => Err(LoadingError::FileSystemError {
                        sub_error: error.into(),
                    }),
                },
                FileSystemOpResult::FileRead(data) => Ok(data),
                _ => panic!("Incorrect file read action response received."),
            }
        }))
    }

    fn read_u32(&self, path: &Path) -> Pin<Box<dyn Future<Output = Result<Vec<u32>, LoadingError>>>> {
        let path = path.to_path_buf();
        let data = self.0.clone();
        Pin::from(Box::new(async move {
            let real_path = {
                let mut p = data.cache.root.clone();
                p.push(path);
                p
            };
            let future = data.reactor.send_async(FileSystemOp::FileReadU32(real_path));

            match future.await {
                FileSystemOpResult::Error(error) => match error.kind() {
                    io::ErrorKind::NotFound => Err(LoadingError::PathNotFound),
                    _ => Err(LoadingError::FileSystemError {
                        sub_error: error.into(),
                    }),
                },
                FileSystemOpResult::FileReadU32(data) => Ok(data),
                _ => panic!("Incorrect file read action response received."),
            }
        }))
    }

    fn read_text(&self, path: &Path) -> Pin<Box<dyn Future<Output = Result<String, LoadingError>>>> {
        let path = path.to_path_buf();
        let data = self.0.clone();
        Pin::from(Box::new(async move {
            let real_path = {
                let mut p = data.cache.root.clone();
                p.push(path);
                p
            };
            let future = data.reactor.send_async(FileSystemOp::FileReadText(real_path));

            match future.await {
                FileSystemOpResult::Error(error) => match error.kind() {
                    io::ErrorKind::NotFound => Err(LoadingError::PathNotFound),
                    _ => Err(LoadingError::FileSystemError {
                        sub_error: error.into(),
                    }),
                },
                FileSystemOpResult::FileReadText(data) => Ok(data),
                _ => panic!("Incorrect file read action response received."),
            }
        }))
    }
}
