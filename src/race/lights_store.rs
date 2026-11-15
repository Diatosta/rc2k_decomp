//! The camera's two place records and the lights' data: the seven dwords `physics::place_camera`
//! copies for a record mode other than the second and the seven for the second, then
//! `files\s\data\ldata` (0x1320 bytes).

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainBytes, PlainDword};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct LightsData {
    /// Seven dwords of the camera's place (`physics::place_camera`, the usual mode).
    pub place_usual: [PlainDword; 7],
    /// Seven dwords of the camera's place (`physics::place_camera`, the second record mode).
    pub place_second: [PlainDword; 7],
    /// The lights' data, 0x22 records of 0x90 bytes (`race::load_lights_data`,
    /// `race::copy_lights_data`).
    pub lights: PlainBytes<0x1320>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with its first values.
impl LightsData {
    pub const fn new() -> Self {
        Self {
            place_usual: [
                PlainDword::new(0x3FA0624E),
                PlainDword::new(0xC0B80000),
                PlainDword::new(0x400FAE14),
                PlainDword::new(0xC0EC0000),
                PlainDword::new(0x404F3B64),
                PlainDword::new(0xC1100000),
                PlainDword::new(0x3C800000),
            ],
            place_second: [
                PlainDword::new(0x3E800000),
                PlainDword::new(0xC0D80000),
                PlainDword::new(0x3EECCCCD),
                PlainDword::new(0xC1060000),
                PlainDword::new(0x3F2CCCCD),
                PlainDword::new(0xC1200000),
                PlainDword::new(0x3CBC6A7F),
            ],
            ..ZeroCells::ZERO
        }
    }
}
