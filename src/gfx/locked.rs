//! The surface-locked switch, out of RAL.EXE's image: IDA 0x4F3870 .. 0x4F3874.

use crate::session::PlainDword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x4F3870;
/// The block's length.
pub const LEN: usize = 4;

/// The surface-locked switch.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Locked {
    /// `g_surfaceLocked` (`dword_4F3870`): 1 while the screen's surface is locked for drawing.
    pub surface: PlainDword,
}

impl Locked {
    /// The switch; the first value is the image's.
    pub const fn new() -> Self {
        Locked {
            surface: PlainDword::new(0),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::gfx::owner::GFX.unchecked().locked,
        );
    }
}
