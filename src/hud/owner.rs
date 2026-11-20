//! The HUD's variables block in one struct, a field of the race.

use crate::hud::block_store;

/// The HUD's state: the one block (`block_store::HudBlock`).
pub struct Hud {
    pub block: block_store::HudBlock,
    /// Whether the clamp of an out-of-range gear has been logged in this race.
    pub gear_clamp_logged: bool,
}

impl Hud {
    pub const fn new() -> Self {
        Self {
            block: block_store::HudBlock::new(),
            gear_clamp_logged: false,
        }
    }
}
