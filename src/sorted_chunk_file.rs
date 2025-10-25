use std::cmp::Ordering;
use std::path::PathBuf;

pub(crate) struct SortedChunkFile {
    path: PathBuf,
    lines: usize,
}

impl SortedChunkFile {
    pub(crate) fn new(path: PathBuf, lines: usize) -> SortedChunkFile {
        SortedChunkFile {
            path,
            lines,
        }
    }

    pub(crate) fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Eq for SortedChunkFile {}

impl PartialEq<Self> for SortedChunkFile {
    fn eq(&self, other: &Self) -> bool {
        self.lines.eq(&other.lines)
    }
}

impl PartialOrd<Self> for SortedChunkFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortedChunkFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.lines.cmp(&other.lines)
    }
}