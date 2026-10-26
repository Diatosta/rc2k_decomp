//! The 3D primary and secondary card switches, out of RAL.EXE's image (phase 5; typed in
//! phase 7, `notes/conversion.md`): IDA 0x5AB4AC .. 0x5AB4B4.

use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x5AB4AC;
/// The block's length.
pub const LEN: usize = 8;

/// Whether a 3D card was found.
#[repr(C)]
pub struct Cards {
    /// `g_primary3dCard` (`dword_5AB4AC`): a 3D primary card.
    pub primary: Dword,
    /// `g_secondary3dCard` (`dword_5AB4B0`): a 3D secondary card.
    pub secondary: Dword,
}

/// The switches; the first values are the image's.
pub static CARDS: Cards = Cards {
    primary: Dword::new(0),
    secondary: Dword::new(0),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x5AB4AC).
pub fn home() -> usize {
    crate::pointers::game_address(&CARDS as *const Cards) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&CARDS, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), CARDS.primary.addr() as usize);
        assert_eq!(
            crate::moved::place(START + 4),
            CARDS.secondary.addr() as usize
        );
    }
}
