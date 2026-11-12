//! The 3D primary and secondary card switches, out of RAL.EXE's image: IDA 0x5AB4AC .. 0x5AB4B4.

use crate::session::PlainDword;

/// Whether a 3D card was found.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Cards {
    /// `g_primary3dCard` (`dword_5AB4AC`): a 3D primary card.
    pub primary: PlainDword,
    /// `g_secondary3dCard` (`dword_5AB4B0`): a 3D secondary card.
    pub secondary: PlainDword,
}

impl Cards {
    /// The switches; the first values are the image's.
    pub const fn new() -> Self {
        Cards {
            primary: PlainDword::new(0),
            secondary: PlainDword::new(0),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::gfx::owner::GFX.get().display.cards,
        );
    }
}
