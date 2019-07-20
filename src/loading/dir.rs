use crate::loading::{FileTree, LoadingError};
use failure::Error;
use futures::Future;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

enum DirectoryEntry {
    Directory(Vec<PathBuf>),
    File,
}

pub struct DirectoryFileTree {
    root: PathBuf,
    file_map: HashMap<PathBuf, DirectoryEntry>,
}

fn enumerate_directories_recursively(path: PathBuf) -> DirectoryEntry {}

impl FileTree<'_> for DirectoryFileTree {
    type CreateResult = Self;
    type DirIter = ();

    fn from_path(path: &Path) -> Box<dyn Future<Output = Result<Self::CreateResult, LoadingError>>> {
        let path = path.to_path_buf();
        Box::new(async move {
            if !path.exists() {
                return Err(LoadingError::PathNotFound);
            }
            if !path.is_dir() {
                return Err(LoadingError::NotDirectory);
            }
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
