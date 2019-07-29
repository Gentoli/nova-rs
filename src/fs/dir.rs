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

impl DirectoryEntry {
    /// Get [`DirectoryEntry`] corresponding with the path given. Path is relative
    /// to the node being called.
    ///
    /// # Example
    ///
    /// ```edition2018,no_run
    /// # use nova_rs::fs::dir::{read_recursive, DirectoryEntry};
    /// # use std::path::Path;
    /// let dir_entry = read_recursive(&"/path/to/some/dir")?;
    ///
    /// // /path/to/some/dir/a/b
    /// let b_entry = dir_entry.entry.get("a/b").unwrap();
    ///
    /// // /path/to/some/dir/a/b/c/d
    /// let d_entry = b_entry.get("c/d").unwrap();
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn get<P>(&self, path: P) -> Option<&DirectoryEntry>
    where
        P: AsRef<Path>,
    {
        let path_iter = path.as_ref().components().peekable();

        let mut node = self;
        for component in path_iter {
            match node {
                DirectoryEntry::File => {
                    return None;
                }
                DirectoryEntry::Directory { entries: map } => {
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
///
/// # Example
///
/// ```edition2018,no_run
/// # use maplit::hashmap;
/// # use nova_rs::fs::dir::{read_recursive, DirectoryEntry};
/// # use std::path::Path;
/// // dir
/// // dir/a
/// // dir/b
/// // dir/c/d
///
/// let value = read_recursive(&"/path/to/dir")?;
/// assert_eq!(value.entry.get("a"), Some(&DirectoryEntry::File));
/// assert_eq!(value.entry.get("b"), Some(&DirectoryEntry::File));
/// assert_eq!(value.entry.get("c"), Some(&DirectoryEntry::Directory { entries: hashmap!{"d".into() => DirectoryEntry::File} }));
///
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn read_recursive<P>(root: P) -> Result<DirectoryTree, io::Error>
where
    P: AsRef<Path>,
{
    let root = std::fs::canonicalize(root)?;
    let entry = read_recursive_impl(&root, Path::new("/"))?;

    Ok(DirectoryTree {
        root: std::fs::canonicalize(root)?,
        entry,
    })
}
