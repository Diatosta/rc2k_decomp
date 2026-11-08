//! The 3D primary and secondary card switches, out of RAL.EXE's image (phase 5; typed in
//! phase 7, `notes/conversion.md`): IDA 0x5AB4AC .. 0x5AB4B4.

use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x5AB4AC;
/// The block's length.
pub const LEN: usize = 8;

/// Whether a 3D card was found.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Cards {
    /// `g_primary3dCard` (`dword_5AB4AC`): a 3D primary card.
    pub primary: Dword,
    /// `g_secondary3dCard` (`dword_5AB4B0`): a 3D secondary card.
    pub secondary: Dword,
}

impl Cards {
    /// The switches; the first values are the image's.
    pub const fn new() -> Self {
        Cards {
            primary: Dword::new(0),
            secondary: Dword::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::gfx::owner::GFX.display.cards,
            START,
            START + LEN,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(
            crate::moved::place(START),
            crate::gfx::owner::GFX.display.cards.primary.addr() as usize
        );
        assert_eq!(
            crate::moved::place(START + 4),
            crate::gfx::owner::GFX.display.cards.secondary.addr() as usize
        );
    }
}
