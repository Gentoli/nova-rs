use crate::fs;
use crate::fs::dir::DirectoryTree;
use std::io;
use std::path::PathBuf;

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
    Error(io::Error),
}

pub fn file_system_reactor_core(op: FileSystemOp) -> FileSystemOpResult {
    match op {
        FileSystemOp::RecursiveEnumerate(path) => match fs::dir::read_recursive(&path) {
            Ok(cache) => FileSystemOpResult::RecursiveEnumerate(cache),
            Err(err) => FileSystemOpResult::Error(err),
        },
        FileSystemOp::FileRead(path) => {
            let file = std::fs::File::open(path);
            match file {
                Ok(reader) => match fs::file::read_stream_u8(reader) {
                    Ok(result) => FileSystemOpResult::FileRead(result),
                    Err(err) => FileSystemOpResult::Error(err),
                },
                Err(err) => FileSystemOpResult::Error(err),
            }
        }
        FileSystemOp::FileReadU32(path) => {
            let file = std::fs::File::open(path);
            match file {
                Ok(reader) => match fs::file::read_stream_u32(reader) {
                    Ok(result) => FileSystemOpResult::FileReadU32(result),
                    Err(err) => FileSystemOpResult::Error(err),
                },
                Err(err) => FileSystemOpResult::Error(err),
            }
        }
        FileSystemOp::FileReadText(path) => {
            let file = std::fs::File::open(path);
            match file {
                Ok(reader) => match fs::file::read_stream_string(reader) {
                    Ok(result) => FileSystemOpResult::FileReadText(result),
                    Err(err) => FileSystemOpResult::Error(err),
                },
                Err(err) => FileSystemOpResult::Error(err),
            }
        }
    }
}
