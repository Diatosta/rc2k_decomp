//! The HUD's variables block in one struct, a field of the race.

use crate::hud::block_store;

/// The HUD's state: the one block (`block_store::HudBlock`).
pub struct Hud {
    pub block: block_store::HudBlock,
}

impl Hud {
    pub const fn new() -> Self {
        Self {
            block: block_store::HudBlock::new(),
        }
    }
}
