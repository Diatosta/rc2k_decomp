//! The HUD's variables block in one struct, a field of the race.

use crate::hud::store;

/// The HUD's state: the one block (`store::HudBlock`).
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

/// What the HUD's polygons are added through: the polygon work of the slot the HUD draws on
/// the system whose screen size the virtual screen is scaled to, and the scene whose texture
/// slots the batch sets.
#[derive(Clone, Copy)]
pub struct Pen<'a> {
    pub polygons: &'a crate::scene::polygon_work::PolygonWork,
    /// The vertex buffer (the game's thread's part's: the HUD draws after the parts).
    pub vertices: &'a crate::d3d::vertices_store::PolygonVertices,
    /// That part, whose render states the HUD's batches set and draw with.
    pub part: &'a crate::workers::PartScratch,
    pub system: &'a crate::system::owner::System,
    pub scene: &'a crate::scene::owner::Scene,
    /// The sine tables the dials turn by (`scene::engine_tables_store`).
    pub engine_tables: &'a crate::scene::engine_tables_store::EngineTables,
}
