use crate::core::reactor::SingleThreadReactor;
use crate::loading::{FileTree, LoadingError};
use futures::Future;
use matches::matches;
use std::collections::hash_map;
use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct DirectoryFileTree(Arc<DirectoryFileTreeData>);

struct DirectoryFileTreeData {
    cache: DirectoryCache,
    reactor: SingleThreadReactor<FileSystemOp, FileSystemOpResult>,
}

struct DirectoryCache {
    root: PathBuf,
    entry: DirectoryEntry,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectoryEntry {
    Directory(HashMap<OsString, DirectoryEntry>),
    File,
}

enum FileSystemOp {
    RecursiveEnumerate(PathBuf),
    FileRead(PathBuf),
    FileReadU32(PathBuf),
    FileReadText(PathBuf),
}

enum FileSystemOpResult {
    RecursiveEnumerate(DirectoryCache),
    FileRead(Vec<u8>),
    FileReadU32(Vec<u32>),
    FileReadText(String),
    Error(io::Error),
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

fn read_stream_u32<R>(mut reader: R) -> Result<Vec<u32>, io::Error>
where
    R: io::Read + io::Seek,
{
    let u32_length = reader.stream_len()? as usize;
    let mut array = Vec::new();
    array.reserve(u32_length);
    let mut tmp = [0_u8; 4];
    while reader.read(&mut tmp)? == 4 {
        array.push(u32::from_le_bytes(tmp));
    }
    Ok(array)
}

fn file_system_reactor_core(op: FileSystemOp) -> FileSystemOpResult {
    match op {
        FileSystemOp::RecursiveEnumerate(path) => match recursive_enumerate_directory(&path) {
            Ok(cache) => FileSystemOpResult::RecursiveEnumerate(cache),
            Err(err) => FileSystemOpResult::Error(err),
        },
        FileSystemOp::FileRead(path) => match std::fs::read(path) {
            Ok(result) => FileSystemOpResult::FileRead(result),
            Err(err) => FileSystemOpResult::Error(err),
        },
        FileSystemOp::FileReadU32(path) => {
            let file = std::fs::File::create(path);
            let buffered = file.map(|f| io::BufReader::with_capacity(64 * 1024, f));
            match buffered {
                Ok(reader) => match read_stream_u32(reader) {
                    Ok(result) => FileSystemOpResult::FileReadU32(result),
                    Err(err) => FileSystemOpResult::Error(err),
                },
                Err(err) => FileSystemOpResult::Error(err),
            }
        }
        FileSystemOp::FileReadText(path) => match std::fs::read_to_string(path) {
            Ok(result) => FileSystemOpResult::FileReadText(result),
            Err(err) => FileSystemOpResult::Error(err),
        },
    }
}
