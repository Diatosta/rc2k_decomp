//! The PNG decoder's file, out of RAL.EXE's image (phase 5; typed in phase 7,
//! `notes/conversion.md`): IDA 0x5BEC30 .. 0x5BEC48. `read_png` reads the file into memory
//! it takes from the list kept here, and gives the list back on every way out.

use crate::memory::MemoryList;
use crate::session::{Dword, Shared};

/// IDA's address of the block's first byte.
pub const START: usize = 0x5BEC30;
/// The block's length.
pub const LEN: usize = 0x5BEC48 - START;

/// The file being decoded.
#[repr(C)]
pub struct PngFile {
    /// `dword_5BEC30`: the file's memory (the game's address; unused since the file's
    /// bytes are Rust's, phase 12).
    pub file: Dword,
    /// `dword_5BEC34`: the file's length.
    pub length: Dword,
    /// `dword_5BEC38`: the list the file's memory is on (16 bytes).
    pub memory: Shared<MemoryList>,
}

/// The file; the first values are the image's.
pub static PNG_FILE: PngFile = PngFile {
    file: Dword::new(0),
    length: Dword::new(0),
    memory: Shared::new(MemoryList {
        head: 0,
        total: 0,
        cursor: 0,
        remaining: 0,
    }),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x5BEC30).
pub fn home() -> usize {
    crate::pointers::game_address(&PNG_FILE as *const PngFile) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&PNG_FILE, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), PNG_FILE.file.addr() as usize);
        assert_eq!(
            crate::moved::place(0x5BEC38),
            PNG_FILE.memory.addr() as usize
        );
    }
}
