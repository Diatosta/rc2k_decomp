//! The polygon vertex buffer (IDA 0x9605A0 .. 0x960DA0): 64 transformed and lit vertices (`TLVertex`, 0x20 bytes), one buffer
//! for each drawing part (`workers::PartScratch::vertices`). The polygon about to be drawn stands in its first vertices
//! (`DRAW.polygon_vertex_count` says how many).

use crate::session::{CellKind, PlainCells};

/// One `TLVertex` of IDA: a Direct3D transformed and lit vertex.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct Vertex<K: CellKind = PlainCells> {
    /// +0x0: the screen x.
    pub x: K::Float,
    /// +0x4: the screen y.
    pub y: K::Float,
    /// +0x8: the depth.
    pub z: K::Float,
    /// +0xC: the reciprocal of the homogeneous w.
    pub rhw: K::Float,
    /// +0x10: the colour (ARGB).
    pub colour: K::Dword,
    /// +0x14: the specular colour (its top byte the fog).
    pub specular: K::Dword,
    /// +0x18: the texture u.
    pub u: K::Float,
    /// +0x1C: the texture v.
    pub v: K::Float,
}

/// The vertices of the polygon being drawn: a drawing part's own (`workers::PartScratch`), on
/// plain cells, reached by the one thread that draws the part.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct PolygonVertices {
    /// +0x0 (0x9605A0): `g_polygonVertices`: `TLVertex[64]` (`d3d::draw_fan_unclipped`, `scene::draw_polygon`, `effects::`, `hud::`, `drive::tyre_parts`).
    pub vertices: [Vertex; 64],
}

impl PolygonVertices {
    /// The buffer, zeroed as in the image.
    pub const fn new() -> Self {
        crate::cells::zeroed_cells()
    }
}

/// The tests' way to the game's thread's buffer (part 0's, `workers::game_scratch`).
#[cfg(test)]
pub struct GameThreadVertices;

#[cfg(test)]
impl core::ops::Deref for GameThreadVertices {
    type Target = PolygonVertices;
    fn deref(&self) -> &PolygonVertices {
        &crate::workers::game_scratch().vertices
    }
}

/// The game's thread's buffer, for the tests.
#[cfg(test)]
pub static VERTICES: GameThreadVertices = GameThreadVertices;
