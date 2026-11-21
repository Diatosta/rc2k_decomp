//! The PNG decoder's file: `read_png` reads the file into memory it takes from the list kept here,
//! and gives the list back on every way out.

use crate::memory::MemoryList;
use std::cell::Cell;

/// The file being decoded.
pub struct PngFile {
    /// The file's memory (the game's address; unused, as the file's bytes are Rust's).
    pub file: Cell<u32>,
    /// The file's length.
    pub length: Cell<u32>,
    /// `PngFile.memory.head`: the list the file's memory is on (16 bytes).
    pub memory: crate::memory::PlainList,
}

impl PngFile {
    /// The file.
    pub const fn new() -> Self {
        PngFile {
            file: Cell::new(0),
            length: Cell::new(0),
            memory: crate::memory::PlainList::new(MemoryList {
                head: 0,
                total: 0,
                cursor: 0,
                remaining: 0,
            }),
        }
    }
}

#[cfg(test)]
impl PngFile {
    /// Every value of the record, in field order: what a test keeps to put back.
    pub fn values(&self) -> Vec<u32> {
        let mut out = vec![self.file.get(), self.length.get()];
        out.extend({
            let l = self.memory.get();
            [l.head, l.total, l.cursor, l.remaining]
        });
        out
    }

    /// The values `values` returned put back.
    pub fn set_values(&self, values: &[u32]) {
        let mut values = values.iter().copied();
        let mut next = || values.next().expect("a value kept for every field");
        self.file.set(next());
        self.length.set(next());
        self.memory.set(rc2k_core::memory::List {
            head: next(),
            total: next(),
            cursor: next(),
            remaining: next(),
        });
    }
}
