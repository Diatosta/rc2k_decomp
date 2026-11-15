//! The clip's and the glow's variables:
//! `ClipScreen` and `GlowState`.

use crate::d3d::vertices_store::Vertex;
use crate::memory::session::{PlainDword, PlainFloat};

/// The clip's screen edges and work vertices.
#[derive(rc2k_derive::Cells)]
pub struct ClipScreen {
    /// The screen's middle x, middle x less one, middle y and middle y less one
    /// (`effects::init_marks` sets them; `effects::clip_quad` clips against the settings' edge).
    pub edges: [PlainFloat; 4],
    /// Six vertices (0x20 bytes each) `effects::clip_quad` builds the clipped quad in.
    pub vertices: [Vertex; 6],
}

const fn zero_vertex() -> Vertex {
    Vertex {
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

impl ClipScreen {
    pub const fn new() -> Self {
        Self {
            edges: [
                PlainFloat::new(400.0),
                PlainFloat::new(399.0),
                PlainFloat::new(300.0),
                PlainFloat::new(299.0),
            ],
            vertices: [const { zero_vertex() }; 6],
        }
    }
}

/// The glow's variables (and the clip's count).
#[derive(rc2k_derive::Cells)]
pub struct GlowState {
    /// 1 = no glow (initially 1).
    pub off: PlainDword,
    /// The glow's place (as the sprite's x, y, z are taken: z, y, x).
    pub place: [PlainFloat; 3],
    /// The vertex count `effects::clip_quad` leaves (nothing reads it).
    pub clip_count: PlainDword,
    /// The glow sprite's angle.
    pub angle: PlainDword,
    /// Added to the place's z and y.
    pub offset: [PlainFloat; 2],
}

impl GlowState {
    pub const fn new() -> Self {
        Self {
            off: PlainDword::new(0x1),
            place: [
                PlainFloat::new(262144.0),
                PlainFloat::new(-65536.0),
                PlainFloat::new(-65536.0),
            ],
            clip_count: PlainDword::new(0x0),
            angle: PlainDword::new(0x0),
            offset: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
        }
    }
}
