//! The marks' loose variables: the status of the last compare (`scene::vector3`, the marks), the
//! pixel shake, the edge scratch, the marks' random seed, the sound pair, the cosine search's
//! constants.

use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells)]
pub struct MarksWork {
    /// The last compare's C0 .. C2 (`scene::vector3`, the marks' compares).
    pub fpu_status: PlainDword,
    /// 1 / 65536, the pixel shake (`effects::pixels`).
    pub shake: PlainFloat,
    /// The scratch of the edge pair (`effects::spans`), the rows' lengths (`effects::turns`) and
    /// the windscreen drop cell's x (`effects::windscreen_drops_frame`).
    pub scratch: PlainDword,
    /// The marks' random seed (times 0x9D plus 0x47 each draw).
    pub seed: PlainDword,
    /// The sound the frame plays (`effects::draw_frame`).
    pub sound_source: PlainDword,
    /// Set to 0 with it (`effects::draw_frame`).
    pub sound_zero: PlainDword,
    /// Pi / 2, the cosine search's start (`scene::vector3`).
    pub cosine_start: PlainDword,
    /// Pi / 4, its step.
    pub cosine_step: PlainDword,
    /// 0.5.
    pub cosine_half: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl MarksWork {
    /// The piece, with the game's first values.
    pub const fn new() -> Self {
        MarksWork {
            fpu_status: PlainDword::new(0),
            shake: PlainFloat::new(1.0 / 65536.0),
            scratch: PlainDword::new(0),
            seed: PlainDword::new(0),
            sound_source: PlainDword::new(0),
            sound_zero: PlainDword::new(0),
            cosine_start: PlainDword::new(core::f32::consts::FRAC_PI_2.to_bits()),
            cosine_step: PlainDword::new(core::f32::consts::FRAC_PI_4.to_bits()),
            cosine_half: PlainDword::new(0.5f32.to_bits()),
        }
    }
}
