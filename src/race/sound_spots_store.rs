//! The face-drawing, sound-band and sound-spot part of the race block: the face kind flags,
//! view-window limits, sound tables and scratch, and the 36 sound spots with their count.

use crate::memory::cells::ZeroCells;

use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct SoundSpotBlock {
    /// The work vector's length (a float).
    pub work_vector_length: PlainFloat,
    /// `face_kind`; the drawing's is the view's (`race::face_work`), this one not used.
    pub face_kind: PlainDword,
    /// The face's unlit byte; the drawing's is the view's (`race::face_work`), this one not
    /// used.
    pub face_unlit: PlainDword,
    /// Which groups to list (`scene::list_object_face`, `race::car_draw`); now each view's
    /// (`CarDrawState::groups_which`); unused.
    pub groups_which: PlainDword,
    /// A flag set to 1, never read.
    pub flag_18dc: PlainDword,
    /// A flag set to 1, never read.
    pub flag_18e0: PlainDword,
    /// A flag tested and set to 1, never read elsewhere.
    pub flag_18e4: PlainDword,
    /// The view window's size less one (`scene::set_view_window`); no longer written or
    /// read.
    pub less_one: [PlainFloat; 2],
    /// The sound by object kind, a 4-bit index (`physics::place_sound_object`).
    pub object_kind_sounds: [PlainDword; 16],
    /// Sound scratch (`physics::car_physics_step`).
    pub sound_scratch_a: PlainDword,
    /// Sound scratch (`physics::place_sound_e1_other`).
    pub sound_scratch_b: PlainDword,
    /// Sound scratch (`physics::car_physics_step`).
    pub sound_scratch_c: PlainDword,
    /// A band factor; never read.
    pub band_scale_0: PlainFloat,
    /// The factor of the wall / ground contact sound (the contact band (`sound_bands::CONTACT_BAND`) for
    /// `place_sound_e1`, called by the wall and ground contacts) and of the object band (`sound_bands::OBJECT_BAND`).
    pub contact_band_scale: PlainFloat,
    /// The factor of the crash sound (the crash band (`sound_bands::CRASH_BAND`) for `place_sound_e5`, called
    /// by `physics::crash_contact`: the car's speed times it, less 1, held to 0 ..= 0x40).
    pub crash_band_scale: PlainFloat,
    /// The factor of the car-bump sound (the bump band (`sound_bands::BUMP_BAND`) for
    /// `place_sound_e1_other`, called by `physics::bump_cars`: times it, less 10, held to 0 ..=
    /// 0x40).
    pub bump_band_scale: PlainFloat,
    /// A band value rounded to a whole number (`physics::sound_bands`,
    /// `physics::place_sound_e1`).
    pub band_scratch: PlainDword,
    /// The faces summed; never read.
    pub eax_face_count: PlainDword,
    /// 36 spots of 0x14 bytes (`race::sound_spots`, `race::frame_draw`).
    pub sound_spots: [PlainDword; 180],
    /// Spots made this frame (`race::sound_spots`).
    pub sound_spot_count: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl SoundSpotBlock {
    pub const fn new() -> Self {
        Self {
            object_kind_sounds: [
                PlainDword::new(0xD7),
                PlainDword::new(0xD7),
                PlainDword::new(0xD8),
                PlainDword::new(0xD9),
                PlainDword::new(0xDA),
                PlainDword::new(0xDB),
                PlainDword::new(0xDC),
                PlainDword::new(0xDD),
                PlainDword::new(0xDE),
                PlainDword::new(0xDF),
                PlainDword::new(0xE0),
                PlainDword::new(0xE1),
                PlainDword::new(0xD7),
                PlainDword::new(0xD7),
                PlainDword::new(0xD7),
                PlainDword::new(0xD7),
            ],
            band_scale_0: PlainFloat::new(5e+01),
            contact_band_scale: PlainFloat::new(5e+01),
            crash_band_scale: PlainFloat::new(5e+01),
            bump_band_scale: PlainFloat::new(2e+02),
            ..ZeroCells::ZERO
        }
    }
}
