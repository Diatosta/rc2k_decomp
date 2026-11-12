//! The face-drawing, sound-band and sound-spot part of the race block: the face kind flags, view-window limits, sound tables and scratch, and the 36 sound spots with their count. Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SoundSpotBlock {
    /// +0x0 (0x6D18C8): `g_workVectorLength` (a float, stored by `fst` / `movss`).
    pub work_vector_length: PlainFloat,
    /// +0x4 (0x6D18CC): `g_faceKind`; the drawing's is the view's (`race::face_work`), this one
    /// not used.
    pub face_kind: PlainDword,
    /// +0x8 (0x6D18D0): `dword_6D18D0`, the face's unlit byte; the drawing's is the view's
    /// (`race::face_work`), this one not used.
    pub face_unlit: PlainDword,
    /// +0xc (0x6D18D4): `g_groupsWhich` (`scene::list_object_face`, `race::car_draw`); now each view's (`CarDrawState::groups_which`); unused.
    pub groups_which: PlainDword,
    /// +0x10 (0x6D18D8): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_10: PlainDword,
    /// +0x14 (0x6D18DC): `dword_6D18DC`: set to 1 by the original (unused by Rust).
    pub flag_18dc: PlainDword,
    /// +0x18 (0x6D18E0): `dword_6D18E0`: set to 1 by the original (a reading, unused by Rust).
    pub flag_18e0: PlainDword,
    /// +0x1c (0x6D18E4): `dword_6D18E4`: tested and set to 1 (a reading, unused by Rust).
    pub flag_18e4: PlainDword,
    /// +0x20 (0x6D18E8): `g_lessOne`, `flt_6D18EC`: the view window's size less one (`scene::set_view_window`), read only by code nothing reaches (0x45BF11); no longer written.
    pub less_one: [PlainFloat; 2],
    /// +0x28 (0x6D18F0): IDA reads and writes it as 64-bit values only in code outside any function (0x45C40E .. 0x45C4CA, dead); the port never reaches it.
    pub unused_28: [PlainDword; 104],
    /// +0x1c8 (0x6D1A90): `dword_6D1A90[]`: sound by object kind, 4-bit index (`physics::place_sound_object`).
    pub object_kind_sounds: [PlainDword; 16],
    /// +0x208 (0x6D1AD0): `dword_6D1AD0` (`physics::car_physics_step`).
    pub sound_scratch_a: PlainDword,
    /// +0x20c (0x6D1AD4): `dword_6D1AD4` (`physics::place_sound_e1_other`).
    pub sound_scratch_b: PlainDword,
    /// +0x210 (0x6D1AD8): `dword_6D1AD8` (`physics::car_physics_step`).
    pub sound_scratch_c: PlainDword,
    /// +0x214 (0x6D1ADC): `flt_6D1ADC`: a band factor used only by the unported `sub_45D0C7` (IDA, `fmul g_bandScale0`); the port never reads it.
    pub band_scale_0: PlainFloat,
    /// +0x218 (0x6D1AE0): `flt_6D1AE0`: the factor of the wall / ground contact sound (`physics::sound_band_1` for `place_sound_e1`, called by the wall and ground contacts) and of `physics::sound_band_2`.
    pub contact_band_scale: PlainFloat,
    /// +0x21c (0x6D1AE4): `flt_6D1AE4`: the factor of the crash sound (`physics::sound_band_4` for `place_sound_e5`, called by `physics::crash_contact`: the car's speed times it, less 1, held to 0 ..= 0x40).
    pub crash_band_scale: PlainFloat,
    /// +0x220 (0x6D1AE8): `flt_6D1AE8`: the factor of the car-bump sound (`physics::sound_band_3` for `place_sound_e1_other`, called by `physics::bump_cars`: `flt_6D3CD0` times it, less 10, held to 0 ..= 0x40).
    pub bump_band_scale: PlainFloat,
    /// +0x224 (0x6D1AEC): `dword_6D1AEC`: fistp target (`physics::sound_bands`, `physics::place_sound_e1`).
    pub band_scratch: PlainDword,
    /// +0x228 (0x6D1AF0): `g_eaxFaceCount`: the faces summed; read by nothing, not kept.
    pub eax_face_count: PlainDword,
    /// +0x22c (0x6D1AF4): `g_soundSpots`: 36 spots of 0x14 bytes (`race::sound_spots`, `race::frame_draw`).
    pub sound_spots: [PlainDword; 180],
    /// +0x4fc (0x6D1DC4): `g_soundSpotCount`: spots made this frame (`race::sound_spots`).
    pub sound_spot_count: PlainDword,
    /// +0x500 (0x6D1DC8): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_500: [PlainDword; 3],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl SoundSpotBlock {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x6D18C8, the first byte of the piece.
pub const START: usize = 0x6D18C8;
/// IDA 0x6D1DD4, the end of the piece.
pub const END: usize = 0x6D1DD4;

const _: () = assert!(END - START == core::mem::size_of::<SoundSpotBlock>());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            race().frame.sound_spots(),
        );
    }
}
