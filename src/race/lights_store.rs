//! The camera's two place records and the lights' data (IDA 0x6077B0 .. 0x608B08; phase 7,
//! typed): the seven dwords `physics::place_camera` copies for a record mode other than the
//! second and the seven for the second, then `files\s\data\ldata` (0x1320 bytes). `#[repr(C)]`
//! with the image's layout, over the race block's bytes (`LIGHTS_DATA`). Names are readings.

use crate::session::{PlainBytes, PlainDword};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LightsData {
    /// +0x0 (0x6077B0): `unk_6077B0`: seven dwords of the camera's place (`physics::place_camera`, the usual mode).
    pub place_usual: [PlainDword; 7],
    /// +0x1c (0x6077CC): `unk_6077CC`: seven dwords of the camera's place (`physics::place_camera`, the second record mode).
    pub place_second: [PlainDword; 7],
    /// +0x38 (0x6077E8): `g_lightsData`: the lights' data, 0x22 records of 0x90 bytes (`race::load_lights_data`, `race::copy_lights_data`).
    pub lights: PlainBytes<0x1320>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with the image's first values.
impl LightsData {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x6077B0, the first byte of the piece.
pub const START: usize = 0x6077B0;
/// IDA 0x608B08, the end of the piece.
pub const END: usize = 0x608B08;

const _: () = assert!(END - START == core::mem::size_of::<LightsData>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    core::ptr::from_ref(crate::race::owner::RACE.course.lights_data()).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            crate::race::owner::RACE.course.lights_data(),
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(LightsData, place_second), 0x1c);
        assert_eq!(offset_of!(LightsData, lights), 0x38);
    }
}
