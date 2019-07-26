use crate::core::reactor::SingleThreadReactor;
use crate::fs::dir::{DirectoryEntry, DirectoryTree};
use crate::loading::{FileTree, LoadingError};
use futures::Future;
use matches::matches;
use std::collections::hash_map;
use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::sync::Arc;

mod iter;
mod reactor;

pub use iter::*;
use reactor::*;

pub struct DirectoryFileTree(Arc<DirectoryFileTreeData>);

struct DirectoryFileTreeData {
    cache: DirectoryTree,
    reactor: SingleThreadReactor<FileSystemOp, FileSystemOpResult>,
}

impl DirectoryFileTree {
    fn get_node_at_location(&self, path: &Path) -> Option<&DirectoryEntry> {
        let path_iter = path.components().peekable();

        let mut node = &self.0.cache.entry;
        for component in path_iter {
            match node {
                DirectoryEntry::File => {
                    return None;
                }
                DirectoryEntry::Directory(map) => {
                    node = match map.get(component.as_os_str()) {
                        Some(v) => v,
                        None => return None,
                    }
                }
            }
        }

        Some(node)
    }
}

impl<'a> FileTree<'a> for DirectoryFileTree {
    type CreateResult = Self;
    type DirIter = DirectoryIterator<'a>;

    fn from_path(path: &Path) -> Box<dyn Future<Output = Result<Self::CreateResult, LoadingError>>> {
        let path = path.to_path_buf();
        Box::new(async move {
            if !path.exists() {
                return Err(LoadingError::PathNotFound);
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
        })
    }

    fn exists(&'a self, path: &Path) -> bool {
        self.get_node_at_location(path).is_some()
    }

    fn is_file(&'a self, path: &Path) -> Option<bool> {
        self.get_node_at_location(path)
            .map(|v| matches!(v, DirectoryEntry::File))
    }

    fn is_dir(&'a self, path: &Path) -> Option<bool> {
        self.get_node_at_location(path)
            .map(|v| matches!(v, DirectoryEntry::Directory(_)))
    }

    fn read_dir(&'a self, path: &Path) -> Result<Self::DirIter, LoadingError> {
        match self.get_node_at_location(path) {
            Some(DirectoryEntry::File) => Err(LoadingError::NotDirectory),
            Some(DirectoryEntry::Directory(map)) => Ok(map.keys().into()),
            None => Err(LoadingError::PathNotFound),
        }
    }

    fn read(&'a self, path: &Path) -> Box<dyn Future<Output = Result<Vec<u8>, LoadingError>>> {
        let path = path.to_path_buf();
        let data = self.0.clone();
        Box::new(async move {
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
        })
    }

    fn read_u32(&'a self, path: &Path) -> Box<dyn Future<Output = Result<Vec<u32>, LoadingError>>> {
        let path = path.to_path_buf();
        let data = self.0.clone();
        Box::new(async move {
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
        })
    }

    fn read_text(&'a self, path: &Path) -> Box<dyn Future<Output = Result<String, LoadingError>>> {
        let path = path.to_path_buf();
        let data = self.0.clone();
        Box::new(async move {
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
        })
    }
}
