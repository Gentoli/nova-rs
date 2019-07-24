use crate::core::reactor::SingleThreadReactor;
use crate::loading::{FileTree, LoadingError};
use futures::Future;
use matches::matches;
use std::collections::hash_map;
use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

enum FileSystemOp {
    RecursiveEnumerate(PathBuf),
}

enum FileSystemOpResult {
    RecursiveEnumerate(DirectoryCache),
    Error(io::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectoryEntry {
    Directory(HashMap<OsString, DirectoryEntry>),
    File,
}

struct DirectoryCache {
    root: PathBuf,
    entry: DirectoryEntry,
}

pub struct DirectoryFileTree {
    cache: DirectoryCache,
    reactor: SingleThreadReactor<FileSystemOp, FileSystemOpResult>,
}

fn recursive_enumerate_directory_impl(root: &Path, relative: &Path) -> Result<DirectoryEntry, io::Error> {
    let real_path = {
        let mut p = root.to_path_buf();
        p.push(relative);
        p
    };

    if real_path.is_file() {
        Ok(DirectoryEntry::File)
    } else {
        let mut map = HashMap::new();
        for entry_result in real_path.read_dir()? {
            let entry = entry_result?;
            let file_name = entry.file_name();
            let new_path = {
                let mut p = relative.to_path_buf();
                p.push(file_name);
                p
            };
            map.insert(entry.file_name(), recursive_enumerate_directory_impl(root, &new_path)?);
        }
        Ok(DirectoryEntry::Directory(map))
    }
}

fn recursive_enumerate_directory(root: &Path) -> Result<DirectoryCache, io::Error> {
    let entry = recursive_enumerate_directory_impl(root, Path::new("/"))?;

    Ok(DirectoryCache {
        root: root.to_path_buf(),
        entry,
    })
}

fn file_system_reactor_core(op: FileSystemOp) -> FileSystemOpResult {
    match op {
        FileSystemOp::RecursiveEnumerate(path) => match recursive_enumerate_directory(&path) {
            Ok(cache) => FileSystemOpResult::RecursiveEnumerate(cache),
            Err(err) => FileSystemOpResult::Error(err),
        },
    }
}

impl DirectoryFileTree {
    fn get_node_at_location(&self, path: &Path) -> Option<&DirectoryEntry> {
        let mut path_itr = path.components().peekable();

        let mut node = &self.cache.entry;
        while let Some(component) = path_itr.next() {
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

        return Some(node);
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
                FileSystemOpResult::RecursiveEnumerate(cache) => Ok(Self { cache, reactor }),
                FileSystemOpResult::Error(err) => Err(LoadingError::FileSystemError { sub_error: err.into() }),
                _ => panic!("Incorrect directory action response received"),
            }
        })
    }

    fn exists(&'a self, path: &Path) -> bool {
        self.get_node_at_location(path).is_some()
    }

    fn is_file(&'a self, path: &Path) -> Option<bool> {
        return self
            .get_node_at_location(path)
            .map(|v| matches!(v, DirectoryEntry::File));
    }

    fn is_dir(&'a self, path: &Path) -> Option<bool> {
        return self
            .get_node_at_location(path)
            .map(|v| matches!(v, DirectoryEntry::Directory(_)));
    }

    fn read_dir(&'a self, path: &Path) -> Result<Self::DirIter, LoadingError> {
        match self.get_node_at_location(path) {
            Some(DirectoryEntry::File) => Err(LoadingError::NotDirectory),
            Some(DirectoryEntry::Directory(map)) => Ok(map.keys().into()),
            None => Err(LoadingError::PathNotFound),
        }
    }

    fn read(&'a self, path: &Path) -> Box<dyn Future<Output = Result<Vec<u8>, LoadingError>>> {
        unimplemented!()
    }

    fn read_u32(&'a self, path: &Path) -> Box<dyn Future<Output = Result<Vec<u32>, LoadingError>>> {
        unimplemented!()
    }

    fn read_text(&'a self, path: &Path) -> Box<dyn Future<Output = Result<String, LoadingError>>> {
        unimplemented!()
    }
}

pub struct DirectoryIterator<'a> {
    sub_iter: hash_map::Keys<'a, std::ffi::OsString, DirectoryEntry>,
}

impl<'a> From<hash_map::Keys<'a, std::ffi::OsString, DirectoryEntry>> for DirectoryIterator<'a> {
    fn from(sub_iter: hash_map::Keys<'a, OsString, DirectoryEntry>) -> Self {
        DirectoryIterator { sub_iter }
    }
}

impl<'a> Iterator for DirectoryIterator<'a> {
    type Item = &'a Path;

    fn next(&mut self) -> Option<Self::Item> {
        self.sub_iter.next().map(|v| Path::new(v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.sub_iter.size_hint()
    }
}
