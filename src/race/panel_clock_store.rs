//! The panel's clock mark (IDA 0x62290C .. 0x622910; phase 7, typed): the scene clock at
//! which the panel's motion began to show (`race::frame_draw::panel_motion` reads it; no
//! code of the game writes it). `#[repr(C)]` with the image's layout, over the race block's
//! bytes (`PANEL_CLOCK`). Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PanelClock {
    /// +0x0 (0x62290C): `dword_62290C`: the scene clock the panel's motion shows from (`race::panel_motion`).
    pub shown_since: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl PanelClock {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x62290C, the first byte of the piece.
pub const START: usize = 0x62290C;
/// IDA 0x622910, the end of the piece.
pub const END: usize = 0x622910;

const _: () = assert!(END - START == core::mem::size_of::<PanelClock>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    core::ptr::from_ref(race().frame.panel_clock()).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            race().frame.panel_clock(),
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(PanelClock, shown_since), 0);
    }
}
