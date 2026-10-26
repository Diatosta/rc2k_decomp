//! The course kinds' first and last records, 0x5CC3C0 .. 0x5CCBF0: `g_kindFirstRecords` and
//! `g_kindLastRecords`, indexed by kind (`physics::ground`, `race::course_data`,
//! `physics::find_course_record`). The course store follows (`race::course_store`).
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct KindRecords {
    /// +0x0 (0x5CC3C0): not named.
    pub unknown_0: [Dword; 9],
    /// +0x24 (0x5CC3E4): `g_kindFirstRecords`: each kind's first record (`physics::ground`, `race::course_data`, `physics::find_course_record`).
    pub first_records: [Dword; 256],
    /// +0x424 (0x5CC7E4): `g_kindLastRecords`: each kind's last record (`physics::ground`, `race::course_data`).
    pub last_records: [Dword; 259],
}

/// The piece: the race block's bytes at IDA 0x5CC3C0 seen as `KindRecords`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = KindRecords;
    fn deref(&self) -> &KindRecords {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host(piece_addr()) as *const KindRecords) }
    }
}

pub static KIND_RECORDS: InRaceBlock = InRaceBlock;

/// IDA 0x5CC3C0, the first byte of the piece.
pub const START: usize = 0x5CC3C0;
/// IDA 0x5CCBF0, the end of the piece.
pub const END: usize = 0x5CCBF0;

const _: () = assert!(END - START == core::mem::size_of::<KindRecords>());

/// Where IDA 0x5CC3C0 is now (in the race block).
pub fn piece_addr() -> u32 {
    crate::race::store::home(START) as u32
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
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(KindRecords, first_records), 0x5CC3E4 - START);
        assert_eq!(offset_of!(KindRecords, last_records), 0x5CC7E4 - START);
        assert_eq!(offset_of!(KindRecords, last_records), 0x5CC7E4 - START);
    }
}
