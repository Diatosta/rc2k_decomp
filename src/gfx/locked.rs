//! The surface-locked switch, out of RAL.EXE's image (phase 5; typed in phase 7,
//! `notes/conversion.md`): IDA 0x4F3870 .. 0x4F3874.

use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x4F3870;
/// The block's length.
pub const LEN: usize = 4;

/// The surface-locked switch.
#[repr(C)]
pub struct Locked {
    /// `g_surfaceLocked` (`dword_4F3870`): 1 while the screen's surface is locked for drawing.
    pub surface: Dword,
}

/// The switch; the first value is the image's.
pub static LOCKED: Locked = Locked {
    surface: Dword::new(0),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x4F3870).
pub fn home() -> usize {
    &LOCKED as *const Locked as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&LOCKED, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), LOCKED.surface.addr());
    }
}
