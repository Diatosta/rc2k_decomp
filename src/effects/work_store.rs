//! The marks' loose variables: the status of the last compare (`scene::vector3`, the marks), the
//! pixel shake, the edge scratch, the marks' random seed, the sound pair, the cosine search's
//! constants.

pub struct MarksWork {
    /// The last compare's C0 .. C2 (`scene::vector3`, the marks' compares).
    pub fpu_status: u32,
    /// 1 / 65536, the pixel shake (`effects::pixels`).
    pub shake: f32,
    /// The scratch of the edge pair (`effects::spans`), the rows' lengths (`effects::turns`) and
    /// the windscreen drop cell's x (`effects::windscreen_drops_frame`).
    pub scratch: u32,
    /// The marks' random seed (times 0x9D plus 0x47 each draw).
    pub seed: u32,
    /// The sound the frame plays (`effects::draw_frame`).
    pub sound_source: u32,
    /// Set to 0 with it (`effects::draw_frame`).
    pub sound_zero: u32,
    /// Pi / 2, the cosine search's start (`scene::vector3`), as bits.
    pub cosine_start: u32,
    /// Pi / 4, its step, as bits.
    pub cosine_step: u32,
    /// 0.5, as bits.
    pub cosine_half: u32,
}

impl MarksWork {
    /// The piece, with the game's first values.
    pub const fn new() -> Self {
        MarksWork {
            fpu_status: 0,
            shake: 1.0 / 65536.0,
            scratch: 0,
            seed: 0,
            sound_source: 0,
            sound_zero: 0,
            cosine_start: core::f32::consts::FRAC_PI_2.to_bits(),
            cosine_step: core::f32::consts::FRAC_PI_4.to_bits(),
            cosine_half: 0.5f32.to_bits(),
        }
    }
}
