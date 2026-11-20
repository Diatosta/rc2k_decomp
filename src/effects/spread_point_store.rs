//! The spread point's random.

use crate::memory::session::PlainDword;

/// The spread point's random.
#[derive(rc2k_derive::Cells)]
pub struct SpreadRandom {
    /// The random (times 0x9D plus 0x47) of `effects::spread_point`.
    pub random: PlainDword,
}

impl SpreadRandom {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SpreadRandom {
            random: PlainDword::new(0x0),
        }
    }
}
