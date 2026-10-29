//! The camera's two place records and the lights' data (IDA 0x6077B0 .. 0x608B08; phase 7,
//! typed): the seven dwords `physics::place_camera` copies for a record mode other than the
//! second and the seven for the second, then `files\s\data\ldata` (0x1320 bytes). `#[repr(C)]`
//! with the image's layout, over the race block's bytes (`LIGHTS_DATA`). Names are readings.

use crate::session::{Bytes, Dword};

#[repr(C)]
pub struct LightsData {
    /// +0x0 (0x6077B0): `unk_6077B0`: seven dwords of the camera's place (`physics::place_camera`, the usual mode).
    pub place_usual: [Dword; 7],
    /// +0x1c (0x6077CC): `unk_6077CC`: seven dwords of the camera's place (`physics::place_camera`, the second record mode).
    pub place_second: [Dword; 7],
    /// +0x38 (0x6077E8): `g_lightsData`: the lights' data, 0x22 records of 0x90 bytes (`race::load_lights_data`, `race::copy_lights_data`).
    pub lights: Bytes<0x1320>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.
unsafe impl crate::pointers::Cells for LightsData {}

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static LIGHTS_DATA: LightsData =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x6077B0, the first byte of the piece.
pub const START: usize = 0x6077B0;
/// IDA 0x608B08, the end of the piece.
pub const END: usize = 0x608B08;

const _: () = assert!(END - START == core::mem::size_of::<LightsData>());

/// Where IDA 0x6077B0 is now.
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&LIGHTS_DATA))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&LIGHTS_DATA, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(LightsData, place_second), 0x6077CC - START);
        assert_eq!(offset_of!(LightsData, lights), 0x6077E8 - START);
    }
}
