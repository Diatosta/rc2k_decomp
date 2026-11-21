//! The polygon vertex buffer: 64 transformed and lit vertices (`Vertex`, 0x20 bytes), one
//! buffer for each drawing part (`scene::scratch_store::PartScratch::vertices`). The polygon about to be
//! drawn stands in its first vertices (`polygon_vertex_count` says how many).

use crate::memory::session::{CellKind, PlainCells};

/// One vertex: a Direct3D transformed and lit vertex.
pub struct Vertex<K: CellKind = PlainCells> {
    /// The screen x.
    pub x: K::Float,
    /// The screen y.
    pub y: K::Float,
    /// The depth.
    pub z: K::Float,
    /// The reciprocal of the homogeneous w.
    pub rhw: K::Float,
    /// The colour (ARGB).
    pub colour: K::Dword,
    /// The specular colour (its top byte the fog).
    pub specular: K::Dword,
    /// The texture u.
    pub u: K::Float,
    /// The texture v.
    pub v: K::Float,
}

impl Vertex {
    /// The vertex's eight 32-bit words, in the order Direct3D takes them (x, y, z, rhw, colour,
    /// specular, u, v).
    pub fn words(&self) -> crate::d3d::Vertex {
        [
            self.x.bits(),
            self.y.bits(),
            self.z.bits(),
            self.rhw.bits(),
            self.colour.get(),
            self.specular.get(),
            self.u.bits(),
            self.v.bits(),
        ]
    }

    /// A vertex of zeros.
    pub const fn new() -> Self {
        use crate::memory::session::{PlainDword, PlainFloat};
        Self {
            x: PlainFloat::new(0.0),
            y: PlainFloat::new(0.0),
            z: PlainFloat::new(0.0),
            rhw: PlainFloat::new(0.0),
            colour: PlainDword::new(0),
            specular: PlainDword::new(0),
            u: PlainFloat::new(0.0),
            v: PlainFloat::new(0.0),
        }
    }
}

/// The vertices of the polygon being drawn: a drawing part's own (`scene::scratch_store::PartScratch`), on
/// plain cells, reached by the one thread that draws the part.
pub struct PolygonVertices {
    /// The 64 vertices (`d3d::draw_fan_unclipped`, `scene::draw_polygon`, `effects::`, `hud::`,
    /// `drive::tyre_parts`).
    pub vertices: [Vertex; 64],
}

impl PolygonVertices {
    /// The buffer, zeroed as in the image.
    pub const fn new() -> Self {
        Self {
            vertices: [const { Vertex::new() }; 64],
        }
    }
}

/// The tests' way to the game's thread's buffer (part 0's of the test thread's scene).
#[cfg(test)]
pub struct GameThreadVertices;

#[cfg(test)]
impl core::ops::Deref for GameThreadVertices {
    type Target = PolygonVertices;
    fn deref(&self) -> &PolygonVertices {
        &crate::owners::Owners::the()
            .scene
            .scratch
            .game_scratch()
            .vertices
    }
}

/// The game's thread's buffer, for the tests.
#[cfg(test)]
pub static VERTICES: GameThreadVertices = GameThreadVertices;
