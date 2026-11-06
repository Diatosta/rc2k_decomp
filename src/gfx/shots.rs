//! The screenshot switch, its lock and the file name's template, out of RAL.EXE's image
//! (phase 5; typed in phase 7, `notes/conversion.md`): IDA 0x4F52B4 .. 0x4F52C4.

use crate::session::{Bytes, Dword};

/// IDA's address of the block's first byte.
pub const START: usize = 0x4F52B4;
/// The block's length.
pub const LEN: usize = 0x4F52C4 - START;

/// The screenshots' variables.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Shots {
    /// `g_screenshotsOn` (`dword_4F52B4`, a `var\ral.cfg` setting, `screensnapshot`).
    pub on: Dword,
    /// `dword_4F52B8`: the lock a screenshot holds.
    pub locked_here: Dword,
    /// `0x4F52BC`: the screenshot file's name, "000.pcx" and its NUL (the number's three
    /// digits are counted up for each file).
    pub name: Bytes<8>,
}

impl Shots {
    /// The screenshots' variables; the first values are the image's.
    pub const fn new() -> Self {
        Shots {
            on: Dword::new(0),
            locked_here: Dword::new(0),
            name: Bytes::new(*b"000.pcx\0"),
        }
    }
}

/// Whether screenshots are on.
pub fn on() -> u32 {
    crate::gfx::owner::gfx().shots.on.get()
}

/// The switch's first byte.
pub fn on_byte() -> u8 {
    crate::gfx::owner::gfx().shots.on.get() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&crate::gfx::owner::GFX.shots, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(
            crate::moved::place(START),
            crate::gfx::owner::GFX.shots.on.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x4F52B8),
            crate::gfx::owner::GFX.shots.locked_here.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x4F52BC),
            crate::gfx::owner::GFX.shots.name.addr() as usize
        );
    }
}
