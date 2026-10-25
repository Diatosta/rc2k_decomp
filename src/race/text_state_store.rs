//! The frame text's state in the race block, 0x5C6604 .. 0x5C6610: `dword_5C6604`, set to 2 by
//! `race::draw_frame_rate` (the text drawing mode, a reading), and the two dwords after it
//! (not read by the game's code).
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct TextState {
    /// +0x0 (0x5C6604): `dword_5C6604`: set to 2 by `race::draw_frame_rate` before the frame-rate text (a reading).
    pub text_mode: Dword,
    /// +0x4 (0x5C6608): not named.
    pub unknown_4: [Dword; 2],
}

/// The piece: the race block's bytes at IDA 0x5C6604 seen as `TextState`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = TextState;
    fn deref(&self) -> &TextState {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(piece_addr() as *const TextState) }
    }
}

pub static TEXT_STATE: InRaceBlock = InRaceBlock;

/// IDA 0x5C6604, the first byte of the piece.
pub const START: usize = 0x5C6604;
/// IDA 0x5C6610, the end of the piece.
pub const END: usize = 0x5C6610;

const _: () = assert!(END - START == core::mem::size_of::<TextState>());

/// Where IDA 0x5C6604 is now (in the race block).
pub fn piece_addr() -> usize {
    crate::race::store::home(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );
        assert_eq!(offset_of!(TextState, text_mode), 0x5C6604 - START);
        assert_eq!(offset_of!(TextState, text_mode), 0x5C6604 - START);
        assert_eq!(offset_of!(TextState, text_mode), 0x5C6604 - START);
    }
}
