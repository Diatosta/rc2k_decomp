//! Module hud's owner (step 11, owned state): the HUD's variables block in one struct,
//! handed by reference to the code that draws the HUD in place of a global store.
//!
//! `hud()` is the boundary: the callers that do not take the owner yet fetch it there and
//! pass it down. It rises as their callers take the owner, until the program's top holds it
//! (`notes/memory_layer_plan.md`, step 11).

use crate::hud::store;

/// The HUD's state: the one block (IDA 0x75B940 .. 0x764F2C, `store::HudBlock`).
pub struct Hud {
    pub block: store::HudBlock,
}

impl Hud {
    pub const fn new() -> Self {
        Self {
            block: store::HudBlock::new(),
        }
    }
}

/// The one instance. Reached through `hud()`.
pub static HUD: crate::session::GameThread<Hud> = crate::session::GameThread::new(Hud::new());

/// The HUD's state, for a caller that does not take it as a parameter yet.
pub fn hud() -> &'static Hud {
    HUD.get()
}

/// What the HUD's polygons are added through: the polygon work of the slot the HUD draws on
/// the system whose screen size the virtual screen is scaled to, and the scene whose texture
/// slots the batch sets.
#[derive(Clone, Copy)]
pub struct Pen<'a> {
    pub polygons: &'a crate::scene::polygon_work::PolygonWork,
    pub system: &'a crate::system::owner::System,
    pub scene: &'a crate::scene::owner::Scene,
}
