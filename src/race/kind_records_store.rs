//! The course kinds' first and last records, 0x5CC3C0 .. 0x5CCBF0: `g_kindFirstRecords` and
//! `g_kindLastRecords`, indexed by kind (`physics::ground`, `race::course_data`,
//! `physics::find_course_record`). The course store follows (`race::course_store`).
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct KindRecords {
    /// +0x0 (0x5CC3C0): IDA touches +0x1c (0x5CC3DC) only in code outside any function (0x421C63 .. 0x421D96, dead); the port never reaches it.
    pub unused_0: [Dword; 9],
    /// +0x24 (0x5CC3E4): `g_kindFirstRecords`: each kind's first record (`physics::ground`, `race::course_data`, `physics::find_course_record`).
    pub first_records: [Dword; 256],
    /// +0x424 (0x5CC7E4): `g_kindLastRecords`: each kind's last record (`physics::ground`, `race::course_data`).
    pub last_records: [Dword; 259],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a field of `race::owner::Race::course`, formerly a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl KindRecords {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x5CC3C0, the first byte of the piece.
pub const START: usize = 0x5CC3C0;
/// IDA 0x5CCBF0, the end of the piece.
pub const END: usize = 0x5CCBF0;

const _: () = assert!(END - START == core::mem::size_of::<KindRecords>());

/// Where IDA 0x5CC3C0 is now (the piece's static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(std::ptr::from_ref(
        &crate::race::owner::RACE.course.kind_records,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::race::owner::RACE.course.kind_records,
            START,
            END,
        );
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(KindRecords, first_records), 0x5CC3E4 - START);
        assert_eq!(offset_of!(KindRecords, last_records), 0x5CC7E4 - START);
        assert_eq!(offset_of!(KindRecords, last_records), 0x5CC7E4 - START);
    }
}
