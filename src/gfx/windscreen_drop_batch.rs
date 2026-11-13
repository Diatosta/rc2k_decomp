//! The marks' windscreen drop batch drawn by d3d (the index and vertex counts, the fans, the
//! records), out of RAL.EXE's image: IDA
//! 0x62423C .. 0x628A44. `effects::shapes` makes the fans (indices) and the records' fixed
//! parts once; `effects::windscreen_drops_top` / `windscreen_drops_frame` fill the records and ask
//! `d3d::draw_marks_windscreen_drops` to draw.

use crate::session::{PlainBytes, PlainDword};

/// The fans' room: 0x40 fans of 4 triangles of 3 indices (words).
pub const FANS_LEN: usize = 0x1800;
/// The records' room: 0x40 records of 0xC0 bytes (six 0x20-byte vertices each).
pub const RECORDS_LEN: usize = 0x3000;

/// The windscreen drop batch.
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
