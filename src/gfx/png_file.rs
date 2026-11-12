//! The PNG decoder's file, out of RAL.EXE's image: IDA 0x5BEC30 .. 0x5BEC48. `read_png` reads the
//! file into memory it takes from the list kept here, and gives the list back on every way out.

use crate::memory::MemoryList;
use crate::session::PlainDword;

/// The file being decoded.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PngFile {
    /// `dword_5BEC30`: the file's memory (the game's address; unused, as the file's
    /// bytes are Rust's).
    pub file: PlainDword,
    /// `dword_5BEC34`: the file's length.
    pub length: PlainDword,
    /// `PngFile.memory.head`: the list the file's memory is on (16 bytes).
    pub memory: crate::memory::ListCells,
}

impl PngFile {
    /// The file; the first values are the image's.
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

#[cfg(test)]
mod tests {

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::gfx::owner::GFX.get().png.file,
        );
    }
}
