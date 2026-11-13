//! The marks' loose variables: the status of the last compare (`scene::vector3`, the marks), the pixel shake, the edge scratch, the marks' random seed, the sound pair, the cosine search's constants. Names are readings unless IDA's.

use crate::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells)]
pub struct MarksWork {
    /// `g_fpuStatus`: the last compare's C0 .. C2 (`scene::vector3`, the marks' compares).
    pub fpu_status: PlainDword,
    /// `flt_71C870`: 1 / 65536, the pixel shake (`effects::pixels`).
    pub shake: PlainFloat,
    /// `dword_71C880`: the scratch of the edge pair (`effects::spans`), the rows' lengths (`effects::turns`) and the windscreen drop cell's x (`effects::windscreen_drops_frame`).
    pub scratch: PlainDword,
    /// `g_marksSeed`: the marks' random seed (times 0x9D plus 0x47 each draw).
    pub seed: PlainDword,
    /// `dword_71C8E2`: the sound the frame plays (`effects::draw_frame`).
    pub sound_source: PlainDword,
    /// `dword_71C8E6`: set to 0 with it (`effects::draw_frame`).
    pub sound_zero: PlainDword,
    /// `flt_71CA36`: pi / 2, the cosine search's start (`scene::vector3`).
    pub cosine_start: PlainDword,
    /// `flt_71CA3A`: pi / 4, its step.
    pub cosine_step: PlainDword,
    /// `flt_71CA3E`: 0.5.
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
