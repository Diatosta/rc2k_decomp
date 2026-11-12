//! The marks' windscreen drop batch drawn by d3d (the index and vertex counts, the fans, the
//! records), out of RAL.EXE's image: IDA
//! 0x62423C .. 0x628A44. `effects::shapes` makes the fans (indices) and the records' fixed
//! parts once; `effects::windscreen_drops_top` / `windscreen_drops_frame` fill the records and ask
//! `d3d::draw_marks_windscreen_drops` to draw.

use crate::session::{PlainBytes, PlainDword};

/// IDA's address of the block's first byte.
pub const START: usize = 0x62423C;
/// The block's length.
pub const LEN: usize = 0x628A44 - START;

/// The fans' room: 0x40 fans of 4 triangles of 3 indices (words).
pub const FANS_LEN: usize = 0x1800;
/// The records' room: 0x40 records of 0xC0 bytes (six 0x20-byte vertices each).
pub const RECORDS_LEN: usize = 0x3000;

/// The windscreen drop batch.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct WindscreenDrops {
    /// `dword_62423C`: the indices to draw.
    pub index_count: PlainDword,
    /// `g_marksWindscreenDropFans` (`0x624240`): the indices (16 bits, a triangle list), the
    /// fans made once.
    pub fans: PlainBytes<FANS_LEN>,
    /// `dword_625A40`: the vertices to draw.
    pub vertex_count: PlainDword,
    /// `unk_625A44`: the batch's vertices (transformed and lit, 0x20 bytes each), as 0x40
    /// records of 0xC0 bytes.
    pub records: PlainBytes<RECORDS_LEN>,
}

impl WindscreenDrops {
    /// The batch; the first values are the image's (all zero).
    pub const fn new() -> Self {
        WindscreenDrops {
            index_count: PlainDword::new(0),
            fans: PlainBytes::new([0; FANS_LEN]),
            vertex_count: PlainDword::new(0),
            records: PlainBytes::new([0; RECORDS_LEN]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::owner::gfx;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &gfx().card.windscreen_drop_batch,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(WindscreenDrops, fans), 0x4);
        assert_eq!(offset_of!(WindscreenDrops, vertex_count), 0x1804);
        assert_eq!(offset_of!(WindscreenDrops, records), 0x1808);
    }
}
