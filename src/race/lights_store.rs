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

/// The piece: the race block's bytes at IDA 0x6077B0 seen as `LightsData`.
pub struct InBlock;

impl core::ops::Deref for InBlock {
    type Target = LightsData;
    fn deref(&self) -> &LightsData {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host_at(piece_addr()) as *const LightsData) }
    }
}

pub static LIGHTS_DATA: InBlock = InBlock;

/// IDA 0x6077B0, the first byte of the piece.
pub const START: usize = 0x6077B0;
/// IDA 0x608B08, the end of the piece.
pub const END: usize = 0x608B08;

const _: () = assert!(END - START == core::mem::size_of::<LightsData>());

/// Where IDA 0x6077B0 is now.
pub fn piece_addr() -> usize {
    crate::moved::place(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );
        assert_eq!(offset_of!(LightsData, place_second), 0x6077CC - START);
        assert_eq!(offset_of!(LightsData, lights), 0x6077E8 - START);
    }
}
