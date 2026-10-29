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

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.
unsafe impl crate::pointers::Cells for PanelClock {}

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static PANEL_CLOCK: PanelClock =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x62290C, the first byte of the piece.
pub const START: usize = 0x62290C;
/// IDA 0x622910, the end of the piece.
pub const END: usize = 0x622910;

const _: () = assert!(END - START == core::mem::size_of::<PanelClock>());

/// Where IDA 0x62290C is now.
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&PANEL_CLOCK))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&PANEL_CLOCK, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(PanelClock, shown_since), 0);
    }
}
