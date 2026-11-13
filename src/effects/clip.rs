//! The clip's and the glow's variables:
//! `ClipScreen` (IDA 0x7687E4 .. 0x7688B4) and `GlowState` (0x93CC70 .. 0x93CCB0).

use crate::d3d::vertices_store::Vertex;
use crate::session::{PlainDword, PlainFloat};

/// The clip's screen edges and work vertices (IDA 0x7687E4 .. 0x7688B4).
#[derive(rc2k_derive::Cells)]
pub struct ClipScreen {
    /// 0x7687E4: `g_marksScreenMiddleX`, `g_marksScreenMiddleXLess`, `g_marksScreenMiddleY`, `g_marksScreenMiddleYLess`: the screen's middle and one less (`effects::init_marks` sets them; `effects::clip_quad` clips against the settings' edge).
    pub edges: [PlainFloat; 4],
    /// 0x7687F4: `g_marksClipVertices`: six `TLVertex`s (0x20 bytes each) `effects::clip_quad` builds the clipped quad in.
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

/// The glow's variables (and the clip's count) (IDA 0x93CC70 .. 0x93CCB0).
#[derive(rc2k_derive::Cells)]
pub struct GlowState {
    /// 0x93CC70: `dword_93CC70`: 1 = no glow (initially 1).
    pub off: PlainDword,
    /// 0x93CC74: `flt_93CC74`, `flt_93CC78`, `flt_93CC7C`: the glow's place (as the sprite's x, y, z are taken: z, y, x).
    pub place: [PlainFloat; 3],
    /// 0x93CC94: `dword_93CC94`: the vertex count `effects::clip_quad` leaves (nothing ported reads it; it sits in the glow's block).
    pub clip_count: PlainDword,
    /// 0x93CCA4: `dword_93CCA4`: the glow sprite's angle.
    pub angle: PlainDword,
    /// 0x93CCA8: `flt_93CCA8`, `flt_93CCAC`: added to the place's z and y.
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
