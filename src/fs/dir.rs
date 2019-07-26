use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

pub struct DirectoryTree {
    pub root: PathBuf,
    pub entry: DirectoryEntry,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectoryEntry {
    Directory(HashMap<OsString, DirectoryEntry>),
    File,
}

fn read_dir_recursive_impl(root: &Path, relative: &Path) -> Result<DirectoryEntry, io::Error> {
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
            map.insert(entry.file_name(), read_dir_recursive_impl(root, &new_path)?);
        }
        Ok(DirectoryEntry::Directory(map))
    }
}

pub fn read_dir_recursive(root: &Path) -> Result<DirectoryTree, io::Error> {
    let entry = read_dir_recursive_impl(root, Path::new("/"))?;

    Ok(DirectoryTree {
        root: root.to_path_buf(),
        entry,
    })
}
