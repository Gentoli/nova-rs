//! Directory reading/writing

use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

/// A fully loaded directory tree. Result of calling [`read_recursive`] on a directory path.
pub struct DirectoryTree {
    /// Root of the directory tree. This is always an absolute path.
    pub root: PathBuf,
    /// Tree of directory entries.
    ///
    /// It is possible for this directory tree to be only a single file, indicating
    /// that [`root`](DirectoryTree::root) is a path to a file.
    pub entry: DirectoryEntry,
}

/// A single directory entry.
#[derive(Debug, Clone, PartialEq)]
pub enum DirectoryEntry {
    /// The entry is a directory.
    Directory {
        /// All entries inside this directory.
        entries: HashMap<OsString, DirectoryEntry>,
    },
    /// The entry is a file.
    File,
}

fn read_recursive_impl(root: &Path, relative: &Path) -> Result<DirectoryEntry, io::Error> {
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
            map.insert(entry.file_name(), read_recursive_impl(root, &new_path)?);
        }
        Ok(DirectoryEntry::Directory { entries: map })
    }
}

/// Reads a given path recursively. Succeeds on both files and directories.
pub fn read_recursive(root: &Path) -> Result<DirectoryTree, io::Error> {
    let root = std::fs::canonicalize(root)?;
    let entry = read_recursive_impl(&root, Path::new("/"))?;

    Ok(DirectoryTree {
        root: std::fs::canonicalize(root)?,
        entry,
    })
}
