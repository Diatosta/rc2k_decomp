//! The lens flare's parts' levels, made once from their whole-number first values.

/// The lens flare's 17 levels: whole numbers at first, floats once `made` (`effects::make_wave_tables`).
pub struct WaveLevels {
    /// Whether the levels have been made into floats.
    pub made: bool,
    /// The levels.
    pub levels: [u32; 17],
}

impl WaveLevels {
    /// The levels' first values.
    pub const fn new() -> Self {
        WaveLevels {
            made: false,
            levels: [
                0x0, 0x9, 0x2B, 0x62, 0xB4, 0xBF, 0xC6, 0xE7, 0x109, 0x139, 0x151, 0x15C, 0x15E,
                0x17A, 0x181, 0x1B2, 0x200,
            ],
        }
    }
}
