//! The `framecounter` setting, 0x5EB100 (`dword_5EB100`): shows the frame rate when not 0
//! (`race::draw_frame_rate`; `settings::read_settings` names it by address, by design).
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct FrameCounter {
    /// +0x0 (0x5EB100): `dword_5EB100`: the `framecounter` setting (`race::draw_frame_rate`).
    pub show: Dword,
}

/// The piece: the race block's bytes at IDA 0x5EB100 seen as `FrameCounter`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = FrameCounter;
    fn deref(&self) -> &FrameCounter {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host_at(piece_addr()) as *const FrameCounter) }
    }
}

pub static FRAME_COUNTER: InRaceBlock = InRaceBlock;

/// IDA 0x5EB100, the first byte of the piece.
pub const START: usize = 0x5EB100;
/// IDA 0x5EB104, the end of the piece.
pub const END: usize = 0x5EB104;

const _: () = assert!(END - START == core::mem::size_of::<FrameCounter>());

/// Where IDA 0x5EB100 is now (in the race block).
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
        assert_eq!(offset_of!(FrameCounter, show), 0x5EB100 - START);
        assert_eq!(offset_of!(FrameCounter, show), 0x5EB100 - START);
        assert_eq!(offset_of!(FrameCounter, show), 0x5EB100 - START);
    }
}
