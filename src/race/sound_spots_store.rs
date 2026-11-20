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
    /// Unused; it only keeps the layout.
    pub unused_10: PlainDword,
    /// A flag set to 1, never read.
    pub flag_18dc: PlainDword,
    /// A flag set to 1, never read.
    pub flag_18e0: PlainDword,
    /// A flag tested and set to 1, never read elsewhere.
    pub flag_18e4: PlainDword,
    /// The view window's size less one (`scene::set_view_window`); no longer written or
    /// read.
    pub less_one: [PlainFloat; 2],
    /// Unused.
    pub unused_28: [PlainDword; 104],
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
    /// Unused; it only keeps the layout.
    pub unused_500: [PlainDword; 3],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl SoundSpotBlock {
    pub const fn new() -> Self {
        Self {
            unused_28: [
                PlainDword::new(0x80000000),
                PlainDword::new(0x80000000),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0x10000),
                PlainDword::new(0x20003),
                PlainDword::new(0x10003),
                PlainDword::new(0x50004),
                PlainDword::new(0x60007),
                PlainDword::new(0x50007),
                PlainDword::new(0x90008),
                PlainDword::new(0xA000B),
                PlainDword::new(0x9000B),
                PlainDword::new(0xD000C),
                PlainDword::new(0xE000F),
                PlainDword::new(0xD000F),
                PlainDword::new(0x110010),
                PlainDword::new(0x120013),
                PlainDword::new(0x110013),
                PlainDword::new(0x150014),
                PlainDword::new(0x160017),
                PlainDword::new(0x150017),
                PlainDword::new(0x190018),
                PlainDword::new(0x1A001B),
                PlainDword::new(0x19001B),
                PlainDword::new(0x1D001C),
                PlainDword::new(0x1E001F),
                PlainDword::new(0x1D001F),
                PlainDword::new(0x210020),
                PlainDword::new(0x220023),
                PlainDword::new(0x210023),
                PlainDword::new(0x250024),
                PlainDword::new(0x260027),
                PlainDword::new(0x250027),
                PlainDword::new(0x290028),
                PlainDword::new(0x2A002B),
                PlainDword::new(0x29002B),
                PlainDword::new(0x2D002C),
                PlainDword::new(0x2E002F),
                PlainDword::new(0x2D002F),
                PlainDword::new(0x310030),
                PlainDword::new(0x320033),
                PlainDword::new(0x310033),
                PlainDword::new(0x350034),
                PlainDword::new(0x360037),
                PlainDword::new(0x350037),
                PlainDword::new(0x390038),
                PlainDword::new(0x3A003B),
                PlainDword::new(0x39003B),
                PlainDword::new(0x3D003C),
                PlainDword::new(0x3E003F),
                PlainDword::new(0x3D003F),
                PlainDword::new(0x410040),
                PlainDword::new(0x420043),
                PlainDword::new(0x410043),
                PlainDword::new(0x450044),
                PlainDword::new(0x460047),
                PlainDword::new(0x450047),
                PlainDword::new(0x490048),
                PlainDword::new(0x4A004B),
                PlainDword::new(0x49004B),
                PlainDword::new(0x4D004C),
                PlainDword::new(0x4E004F),
                PlainDword::new(0x4D004F),
                PlainDword::new(0x510050),
                PlainDword::new(0x520053),
                PlainDword::new(0x510053),
                PlainDword::new(0x550054),
                PlainDword::new(0x560057),
                PlainDword::new(0x550057),
                PlainDword::new(0x590058),
                PlainDword::new(0x5A005B),
                PlainDword::new(0x59005B),
                PlainDword::new(0x5D005C),
                PlainDword::new(0x5E005F),
                PlainDword::new(0x5D005F),
                PlainDword::new(0x610060),
                PlainDword::new(0x620063),
                PlainDword::new(0x610063),
                PlainDword::new(0x650064),
                PlainDword::new(0x660067),
                PlainDword::new(0x650067),
                PlainDword::new(0x690068),
                PlainDword::new(0x6A006B),
                PlainDword::new(0x69006B),
                PlainDword::new(0x6D006C),
                PlainDword::new(0x6E006F),
                PlainDword::new(0x6D006F),
                PlainDword::new(0x710070),
                PlainDword::new(0x720073),
                PlainDword::new(0x710073),
                PlainDword::new(0x750074),
                PlainDword::new(0x760077),
                PlainDword::new(0x750077),
                PlainDword::new(0x790078),
                PlainDword::new(0x7A007B),
                PlainDword::new(0x79007B),
                PlainDword::new(0x7D007C),
                PlainDword::new(0x7E007F),
                PlainDword::new(0x7D007F),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
            ],
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
