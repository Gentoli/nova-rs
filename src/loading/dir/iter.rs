use crate::fs::dir::DirectoryEntry;
use std::collections::hash_map;
use std::ffi::OsString;
use std::path::Path;

/// Iterator over the contents of directory provided by [`DirectoryFileTree`](crate::loading::DirectoryFileTree)
pub struct DirectoryIterator<'a> {
    sub_iter: hash_map::Keys<'a, OsString, DirectoryEntry>,
}

impl<'a> From<hash_map::Keys<'a, OsString, DirectoryEntry>> for DirectoryIterator<'a> {
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
