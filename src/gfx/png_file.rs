//! The PNG decoder's file: `read_png` reads the file into memory it takes from the list kept here,
//! and gives the list back on every way out.

use crate::memory::MemoryList;
use crate::memory::session::PlainDword;

/// The file being decoded.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct PngFile {
    /// The file's memory (the game's address; unused, as the file's bytes are Rust's).
    pub file: PlainDword,
    /// The file's length.
    pub length: PlainDword,
    /// `PngFile.memory.head`: the list the file's memory is on (16 bytes).
    pub memory: crate::memory::ListCells,
}

impl PngFile {
    /// The file.
    pub const fn new() -> Self {
        PngFile {
            file: PlainDword::new(0),
            length: PlainDword::new(0),
            memory: crate::memory::ListCells::new(MemoryList {
                head: 0,
                total: 0,
                cursor: 0,
                remaining: 0,
            }),
        }
    }
}
