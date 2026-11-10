//! The PNG decoder's file, out of RAL.EXE's image: IDA 0x5BEC30 .. 0x5BEC48. `read_png` reads the
//! file into memory it takes from the list kept here, and gives the list back on every way out.

use crate::memory::MemoryList;
use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x5BEC30;
/// The block's length.
pub const LEN: usize = 0x5BEC48 - START;

/// The file being decoded.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PngFile {
    /// `dword_5BEC30`: the file's memory (the game's address; unused since the file's
    /// bytes are Rust's).
    pub file: Dword,
    /// `dword_5BEC34`: the file's length.
    pub length: Dword,
    /// `dword_5BEC38`: the list the file's memory is on (16 bytes).
    pub memory: crate::memory::ListCells,
}

impl PngFile {
    /// The file; the first values are the image's.
    pub const fn new() -> Self {
        PngFile {
            file: Dword::new(0),
            length: Dword::new(0),
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
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::gfx::owner::GFX.png.file,
            START,
            START + LEN,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(
            crate::moved::place(START),
            crate::gfx::owner::GFX.png.file.file.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x5BEC38),
            crate::gfx::owner::GFX.png.file.memory.addr() as usize
        );
    }
}
