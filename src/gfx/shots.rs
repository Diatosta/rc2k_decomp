//! The screenshot switch, its lock and the file name's template, out of RAL.EXE's image: IDA
//! 0x4F52B4 .. 0x4F52C4.

use crate::gfx::owner::gfx;
use crate::session::{PlainBytes, PlainDword};

/// IDA's address of the block's first byte.
pub const START: usize = 0x4F52B4;
/// The block's length.
pub const LEN: usize = 0x4F52C4 - START;

/// The screenshots' variables.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Shots {
    /// `g_screenshotsOn` (`dword_4F52B4`, a `var\ral.cfg` setting, `screensnapshot`).
    pub on: PlainDword,
    /// `dword_4F52B8`: the lock a screenshot holds.
    pub locked_here: PlainDword,
    /// `0x4F52BC`: the screenshot file's name, "000.pcx" and its NUL (the number's three
    /// digits are counted up for each file).
    pub name: PlainBytes<8>,
}

impl Shots {
    /// The screenshots' variables; the first values are the image's.
    pub const fn new() -> Self {
        Shots {
            on: PlainDword::new(0),
            locked_here: PlainDword::new(0),
            name: PlainBytes::new(*b"000.pcx\0"),
        }
    }
}

/// Whether screenshots are on.
pub fn on() -> u32 {
    gfx().shots.on.get()
}

/// The switch's first byte.
pub fn on_byte(gfx: &crate::gfx::owner::Gfx) -> u8 {
    gfx.shots.on.get() as u8
}

#[cfg(test)]
mod tests {

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::gfx::owner::GFX.unchecked().shots,
        );
    }
}
