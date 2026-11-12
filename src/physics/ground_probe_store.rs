//! The ground probe's marks (IDA 0x620458 .. 0x620474; phase 7, typed): the edge test's
//! product (a qword) and the mark that tells the ground search how it was called.
//! `#[repr(C)]` with the image's layout, over the race block's bytes (`GROUND_PROBE`).
//! Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct GroundProbe {
    /// +0x0 (0x620458): `dword_620458`: referenced only by dead code outside any function (0x450330 ..); the port never reaches it.
    pub unused_0: Dword,
    /// +0x4 (0x62045C): `dword_62045C`: referenced only by dead code outside any function (0x450330 ..); the port never reaches it.
    pub unused_4: Dword,
    /// +0x8 (0x620460): `qword_620460`: the product of the edge test, low then high dword (`physics::crossed_edge_contact`, `physics::ground`, `race::car_shadow`).
    pub product: [Dword; 2],
    /// +0x10 (0x620468): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_10: [Dword; 2],
    /// +0x18 (0x620470): `dword_620470`: 0x40 or 0x80 by the caller of the ground search (`physics::count_ground_probe`, `physics::move_slot`, `physics::probe_ground`, `race::car_starts`).
    pub settle_mark: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl GroundProbe {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x620458, the first byte of the piece.
pub const START: usize = 0x620458;
/// IDA 0x620474, the end of the piece.
pub const END: usize = 0x620474;

const _: () = assert!(END - START == core::mem::size_of::<GroundProbe>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    core::ptr::from_ref(&race().physics.ground_probe).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &race().physics.ground_probe,
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(GroundProbe, product), 0x8);
        assert_eq!(offset_of!(GroundProbe, settle_mark), 0x18);
    }
}
