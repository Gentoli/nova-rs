use crate::core::reactor::SingleThreadReactor;
use crate::loading::{FileTree, LoadingError};
use failure::Error;
use futures::Future;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::slice;

enum FileSystemOp {
    RecursiveEnumerate(PathBuf),
}

enum FileSystemOpResult {
    RecursiveEnumerate(DirectoryCache),
}

pub enum DirectoryEntry {
    Directory(Vec<PathBuf>),
    File,
}

struct DirectoryCache {
    root: PathBuf,
    file_map: HashMap<PathBuf, DirectoryEntry>,
}

pub struct DirectoryFileTree {
    cache: DirectoryCache,
    reactor: SingleThreadReactor<FileSystemOp, FileSystemOpResult>,
}

fn enumerate_directories_recursively(path: PathBuf) -> DirectoryCache {
    unimplemented!()
}

fn file_system_reactor_core(op: FileSystemOp) -> FileSystemOpResult {
    match op {
        FileSystemOp::RecursiveEnumerate(path) => {
            FileSystemOpResult::RecursiveEnumerate(enumerate_directories_recursively(path))
        }
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

            let FileSystemOpResult::RecursiveEnumerate(cache) = future.await;

            Ok(Self { cache, reactor })
        })
    }

    fn exists(&self, path: &Path) -> bool {
        unimplemented!()
    }

    fn is_file(&self, path: &Path) -> Option<bool> {
        unimplemented!()
    }

    fn is_dir(&self, path: &Path) -> Option<bool> {
        unimplemented!()
    }

    fn read_dir(&self, path: &Path) -> Result<Self::DirIter, Error> {
        unimplemented!()
    }

    fn read(&self, path: &Path) -> Box<dyn Future<Output = Result<Vec<u8>, Error>>> {
        unimplemented!()
    }

    fn read_u32(&self, path: &Path) -> Box<dyn Future<Output = Result<Vec<u32>, Error>>> {
        unimplemented!()
    }

    fn read_text(&self, path: &Path) -> Box<dyn Future<Output = Result<String, Error>>> {
        unimplemented!()
    }
}

pub struct DirectoryIterator<'a> {
    subiter: slice::Iter<'a, PathBuf>,
}

impl<'a> Iterator for DirectoryIterator<'a> {
    type Item = &'a Path;

    fn next(&mut self) -> Option<Self::Item> {
        self.subiter.next().map(|v| v.as_path())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.subiter.size_hint()
    }
}
