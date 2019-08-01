use crate::fs;
use crate::fs::dir::DirectoryTree;
use failure::{Backtrace, Fail};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum FileSystemOp {
    RecursiveEnumerate(PathBuf),
    FileRead(PathBuf),
    FileReadU32(PathBuf),
    FileReadText(PathBuf),
}

pub enum FileSystemOpResult {
    RecursiveEnumerate(DirectoryTree),
    FileRead(Vec<u8>),
    FileReadU32(Vec<u32>),
    FileReadText(String),
    Error(FileSystemOpError),
}

#[derive(Fail, Debug)]
#[fail(display = "Filesystem error: {:?} on operation {:?}", error, operation)]
pub struct FileSystemOpError {
    #[fail(cause)]
    pub error: io::Error,
    operation: FileSystemOp,
    backtrace: Backtrace,
}

impl FileSystemOpError {
    fn from_path(error: io::Error, operation: FileSystemOp) -> Self {
        FileSystemOpError {
            error,
            operation,
            backtrace: Backtrace::new(),
        }
    }
}

/// Core operation of the file system reactor
pub(in crate::loading::dir) fn file_system_reactor_core(op: FileSystemOp) -> FileSystemOpResult {
    match &op {
        FileSystemOp::RecursiveEnumerate(path) => match fs::dir::read_recursive(path) {
            Ok(cache) => FileSystemOpResult::RecursiveEnumerate(cache),
            Err(err) => FileSystemOpResult::Error(FileSystemOpError::from_path(err, op.clone())),
        },
        FileSystemOp::FileRead(path) => {
            let file = std::fs::File::open(path);
            match file {
                Ok(reader) => match fs::file::read_stream_u8(reader) {
                    Ok(result) => FileSystemOpResult::FileRead(result),
                    Err(err) => FileSystemOpResult::Error(FileSystemOpError::from_path(err, op.clone())),
                },
                Err(err) => FileSystemOpResult::Error(FileSystemOpError::from_path(err, op.clone())),
            }
        }
        FileSystemOp::FileReadU32(path) => {
            let file = std::fs::File::open(path);
            match file {
                Ok(reader) => match fs::file::read_stream_u32(reader) {
                    Ok(result) => FileSystemOpResult::FileReadU32(result),
                    Err(err) => FileSystemOpResult::Error(FileSystemOpError::from_path(err, op.clone())),
                },
                Err(err) => FileSystemOpResult::Error(FileSystemOpError::from_path(err, op.clone())),
            }
        }
        FileSystemOp::FileReadText(path) => {
            let file = std::fs::File::open(path);
            match file {
                Ok(reader) => match fs::file::read_stream_string(reader) {
                    Ok(result) => FileSystemOpResult::FileReadText(result),
                    Err(err) => FileSystemOpResult::Error(FileSystemOpError::from_path(err, op.clone())),
                },
                Err(err) => FileSystemOpResult::Error(FileSystemOpError::from_path(err, op.clone())),
            }
        }
    }
}
