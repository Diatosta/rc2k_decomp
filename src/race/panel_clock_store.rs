//! The panel's clock mark (IDA 0x62290C .. 0x622910; phase 7, typed): the scene clock at
//! which the panel's motion began to show (`race::frame_draw::panel_motion` reads it; no
//! code of the game writes it). `#[repr(C)]` with the image's layout, over the race block's
//! bytes (`PANEL_CLOCK`). Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct PanelClock {
    /// +0x0 (0x62290C): `dword_62290C`: the scene clock the panel's motion shows from (`race::panel_motion`; a reading).
    pub shown_since: Dword,
}

/// The piece: the race block's bytes at IDA 0x62290C seen as `PanelClock`.
pub struct InBlock;

impl core::ops::Deref for InBlock {
    type Target = PanelClock;
    fn deref(&self) -> &PanelClock {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); the field is an atomic, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host_at(piece_addr()) as *const PanelClock) }
    }
}

pub static PANEL_CLOCK: InBlock = InBlock;

/// IDA 0x62290C, the first byte of the piece.
pub const START: usize = 0x62290C;
/// IDA 0x622910, the end of the piece.
pub const END: usize = 0x622910;

const _: () = assert!(END - START == core::mem::size_of::<PanelClock>());

/// Where IDA 0x62290C is now.
pub fn piece_addr() -> usize {
    crate::moved::place(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );
        assert_eq!(offset_of!(PanelClock, shown_since), 0);
    }
}
