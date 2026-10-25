//! The marks' hexagon batch drawn by d3d (the index and vertex counts, the fans, the
//! records), out of RAL.EXE's image (phase 5; typed in phase 7, `notes/conversion.md`): IDA
//! 0x62423C .. 0x628A44. `marks::shapes` makes the fans (indices) and the records' fixed
//! parts once; `marks::hexagons_top` / `hexagons_frame` fill the records and ask
//! `d3d::draw_marks_hexagons` to draw.

use crate::session::{Bytes, Dword};

/// IDA's address of the block's first byte.
pub const START: usize = 0x62423C;
/// The block's length.
pub const LEN: usize = 0x628A44 - START;

/// The fans' room: 0x40 fans of 4 triangles of 3 indices (words).
pub const FANS_LEN: usize = 0x1800;
/// The records' room: 0x40 records of 0xC0 bytes (six 0x20-byte vertices each).
pub const RECORDS_LEN: usize = 0x3000;

/// The hexagon batch.
#[repr(C)]
pub struct Hexagons {
    /// `dword_62423C`: the indices to draw.
    pub index_count: Dword,
    /// `g_marksHexagonFans` (`0x624240`): the indices (16 bits, a triangle list), the
    /// fans made once.
    pub fans: Bytes<FANS_LEN>,
    /// `dword_625A40`: the vertices to draw.
    pub vertex_count: Dword,
    /// `unk_625A44`: the batch's vertices (transformed and lit, 0x20 bytes each), as 0x40
    /// records of 0xC0 bytes.
    pub records: Bytes<RECORDS_LEN>,
}

/// The batch; the first values are the image's (all zero).
pub static HEXAGON_BATCH: Hexagons = Hexagons {
    index_count: Dword::new(0),
    fans: Bytes::new([0; FANS_LEN]),
    vertex_count: Dword::new(0),
    records: Bytes::new([0; RECORDS_LEN]),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x62423C).
pub fn home() -> usize {
    &HEXAGON_BATCH as *const Hexagons as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&HEXAGON_BATCH, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Hexagons, fans), 0x624240 - START);
        assert_eq!(offset_of!(Hexagons, vertex_count), 0x625A40 - START);
        assert_eq!(offset_of!(Hexagons, records), 0x625A44 - START);
        assert_eq!(crate::moved::place(START), HEXAGON_BATCH.index_count.addr());
        assert_eq!(crate::moved::place(0x624240), HEXAGON_BATCH.fans.addr());
        assert_eq!(crate::moved::place(0x625A44), HEXAGON_BATCH.records.addr());
    }
}
