//! The windscreen drop batch drawn by d3d (the index and vertex counts, the fans, the records).
//! `effects::shapes` makes the fans (indices) and the records' fixed parts once;
//! `effects::windscreen_drops_top` / `windscreen_drops_frame` fill the records and ask
//! `d3d::draw_marks_windscreen_drops` to draw.

use crate::memory::plain_buffer::PlainBuffer;
use std::cell::Cell;

/// The fans' room: 0x40 fans of 4 triangles of 3 indices (words).
pub const FANS_LEN: usize = 0x1800;
/// The records' room: 0x40 records of 0xC0 bytes (six 0x20-byte vertices each).
pub const RECORDS_LEN: usize = 0x3000;

/// The windscreen drop batch.
pub struct WindscreenDrops {
    /// The indices to draw.
    pub index_count: Cell<u32>,
    /// The indices (16 bits, a triangle list), the fans made once.
    pub fans: PlainBuffer<FANS_LEN>,
    /// The vertices to draw.
    pub vertex_count: Cell<u32>,
    /// The batch's vertices (transformed and lit, 0x20 bytes each), as 0x40 records of 0xC0 bytes.
    pub records: PlainBuffer<RECORDS_LEN>,
}

impl WindscreenDrops {
    /// The batch.
    pub const fn new() -> Self {
        WindscreenDrops {
            index_count: Cell::new(0),
            fans: PlainBuffer::new([0; FANS_LEN]),
            vertex_count: Cell::new(0),
            records: PlainBuffer::new([0; RECORDS_LEN]),
        }
    }
}
