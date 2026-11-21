//! What the HUD draws through: the borrows the drawing functions are handed, and the part of the
//! screen a split screen draws into.

/// What the HUD's polygons are added through: the polygon work of the slot the HUD draws on
/// the system whose screen size the virtual screen is scaled to, and the scene whose texture
/// slots the batch sets.
#[derive(Clone, Copy)]
pub struct Pen<'a> {
    pub polygons: &'a crate::scene::polygon_work_store::PolygonWork,
    /// The vertex buffer (the game's thread's part's: the HUD draws after the parts).
    pub vertices: &'a crate::d3d::vertices_store::PolygonVertices,
    /// That part, whose render states the HUD's batches set and draw with.
    pub part: &'a crate::scene::scratch_store::PartScratch,
    pub system: &'a crate::system::owner::System,
    pub scene: &'a crate::scene::owner::Scene,
    /// The sine tables the dials turn by (`math::engine_tables`).
    pub engine_tables: crate::math::engine_tables::EngineTablesRef<'a>,
    /// The part of a split screen being drawn into (none: the whole screen).
    pub scope: PartScope,
}

/// The part of the screen the HUD is being drawn into. A split screen of three or four players
/// draws each player's HUD as if the part were a screen of its own: the pieces keep to the
/// part's corners and middle by its shape, and are put where the part is.
#[derive(Clone, Copy, Default)]
pub struct PartScope {
    /// The part's corner and size, if one is set.
    pub part: Option<rc2k_core::race::view::PlayerPart>,
    /// The player whose half-size car icon is drawn (none: the first full-size icon).
    pub icon: Option<usize>,
}
