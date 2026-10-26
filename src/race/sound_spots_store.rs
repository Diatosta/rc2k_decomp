//! The face-drawing, sound-band and sound-spot part of the race block: the face kind flags, view-window limits, sound tables and scratch, and the 36 sound spots with their count. Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
pub struct SoundSpotBlock {
    /// +0x0 (0x6D18C8): `g_workVectorLength` (a reading: float, stored by fst/movss).
    pub work_vector_length: Float,
    /// +0x4 (0x6D18CC): `g_faceKind` (`scene::draw_object_face`, `scene::draw_object_faces`).
    pub face_kind: Dword,
    /// +0x8 (0x6D18D0): `dword_6D18D0` (`scene::draw_object_face`, `scene::draw_object_faces`).
    pub face_unlit: Dword,
    /// +0xc (0x6D18D4): `dword_6D18D4` (`scene::list_object_face`, `race::car_draw`).
    pub groups_which: Dword,
    /// +0x10 (0x6D18D8): not named.
    pub unknown_10: Dword,
    /// +0x14 (0x6D18DC): `dword_6D18DC`: set to 1 by the original (a reading, unused by Rust).
    pub flag_18dc: Dword,
    /// +0x18 (0x6D18E0): `dword_6D18E0`: set to 1 by the original (a reading, unused by Rust).
    pub flag_18e0: Dword,
    /// +0x1c (0x6D18E4): `dword_6D18E4`: tested and set to 1 (a reading, unused by Rust).
    pub flag_18e4: Dword,
    /// +0x20 (0x6D18E8): `flt_6D18E8`, `flt_6D18EC`: view-window limits (`scene::set_view_window`).
    pub less_one: [Float; 2],
    /// +0x28 (0x6D18F0): not named.
    pub unknown_28: [Dword; 104],
    /// +0x1c8 (0x6D1A90): `dword_6D1A90[]`: sound by object kind, 4-bit index (`physics::place_sound_object`).
    pub object_kind_sounds: [Dword; 16],
    /// +0x208 (0x6D1AD0): `dword_6D1AD0` (`physics::car_physics_step`).
    pub sound_scratch_a: Dword,
    /// +0x20c (0x6D1AD4): `dword_6D1AD4` (`physics::place_sound_e1_other`).
    pub sound_scratch_b: Dword,
    /// +0x210 (0x6D1AD8): `dword_6D1AD8` (`physics::car_physics_step`).
    pub sound_scratch_c: Dword,
    /// +0x214 (0x6D1ADC): `flt_6D1ADC` (a reading, sound band factor).
    pub band_scale_0: Float,
    /// +0x218 (0x6D1AE0): `flt_6D1AE0` (`physics::sound_band_1`, `physics::sound_band_2`).
    pub band_scale_1: Float,
    /// +0x21c (0x6D1AE4): `flt_6D1AE4` (`physics::sound_band_4`).
    pub band_scale_2: Float,
    /// +0x220 (0x6D1AE8): `flt_6D1AE8` (`physics::sound_band_3`).
    pub band_scale_3: Float,
    /// +0x224 (0x6D1AEC): `dword_6D1AEC`: fistp target (`physics::sound_bands`, `physics::place_sound_e1`).
    pub band_scratch: Dword,
    /// +0x228 (0x6D1AF0): `g_eaxFaceCount`: the faces summed (`race::frame_draw`).
    pub eax_face_count: Dword,
    /// +0x22c (0x6D1AF4): `g_soundSpots`: 36 spots of 0x14 bytes (`race::sound_spots`, `race::frame_draw`).
    pub sound_spots: [Dword; 180],
    /// +0x4fc (0x6D1DC4): `g_soundSpotCount`: spots made this frame (`race::sound_spots`).
    pub sound_spot_count: Dword,
    /// +0x500 (0x6D1DC8): not named.
    pub unknown_500: [Dword; 3],
}

/// The piece: the race block's bytes at IDA 0x6D18C8 seen as `SoundSpotBlock`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = SoundSpotBlock;
    fn deref(&self) -> &SoundSpotBlock {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host(piece_addr()) as *const SoundSpotBlock) }
    }
}

pub static RACE_SOUND_SPOTS: InRaceBlock = InRaceBlock;

/// IDA 0x6D18C8, the first byte of the piece.
pub const START: usize = 0x6D18C8;
/// IDA 0x6D1DD4, the end of the piece.
pub const END: usize = 0x6D1DD4;

const _: () = assert!(END - START == core::mem::size_of::<SoundSpotBlock>());

/// Where IDA 0x6D18C8 is now (in the race block).
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
            offset_of!(SoundSpotBlock, work_vector_length),
            0x6D18C8 - START
        );
        assert_eq!(offset_of!(SoundSpotBlock, flag_18e4), 0x6D18E4 - START);
        assert_eq!(offset_of!(SoundSpotBlock, band_scale_0), 0x6D1ADC - START);
        assert_eq!(offset_of!(SoundSpotBlock, sound_spots), 0x6D1AF4 - START);
    }
}
