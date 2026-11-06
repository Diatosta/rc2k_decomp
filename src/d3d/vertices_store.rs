//! The polygon vertex buffer `g_polygonVertices`, typed (phase 7): IDA 0x9605A0 .. 0x960DA0,
//! 64 transformed and lit vertices (`TLVertex`, 0x20 bytes) in the image's zeroed tail
//! (`race::buffers_store`). The polygon about to be drawn stands in its first vertices
//! (`g_polygonVertexCount`, `DRAW.polygon_vertex_count`, says how many).

use crate::session::{Dword, Float};

/// One `TLVertex` of IDA: a Direct3D transformed and lit vertex.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Vertex {
    /// +0x0: the screen x.
    pub x: Float,
    /// +0x4: the screen y.
    pub y: Float,
    /// +0x8: the depth.
    pub z: Float,
    /// +0xC: the reciprocal of the homogeneous w.
    pub rhw: Float,
    /// +0x10: the colour (ARGB).
    pub colour: Dword,
    /// +0x14: the specular colour (its top byte the fog).
    pub specular: Dword,
    /// +0x18: the texture u.
    pub u: Float,
    /// +0x1C: the texture v.
    pub v: Float,
}

/// The vertices of the polygon being drawn.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PolygonVertices {
    /// +0x0 (0x9605A0): `g_polygonVertices`: `TLVertex[64]` (`d3d::draw_fan_unclipped`, `scene::draw_polygon`, `effects::`, `hud::`, `drive::tyre_parts`).
    pub vertices: [Vertex; 64],
}

/// The piece, a field of `crate::race::owner::race().shared.buffers`.
pub struct InBuffers;

impl core::ops::Deref for InBuffers {
    type Target = PolygonVertices;
    fn deref(&self) -> &PolygonVertices {
        &crate::race::owner::race().shared.buffers.vertices
    }
}

pub static VERTICES: InBuffers = InBuffers;

/// IDA 0x9605A0, the first byte of the piece.
pub const START: usize = 0x9605A0;
/// IDA 0x960DA0, the end of the piece.
pub const END: usize = 0x960DA0;

const _: () = assert!(END - START == core::mem::size_of::<PolygonVertices>());

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    fn piece_addr() -> u32 {
        crate::pointers::game_address(core::ptr::from_ref(&*VERTICES))
    }

    #[test]
    fn piece_sits_on_the_buffers() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(PolygonVertices, vertices), 0);
        assert_eq!(offset_of!(Vertex, colour), 0x10);
        assert_eq!(offset_of!(Vertex, v), 0x1C);
        assert_eq!(core::mem::size_of::<Vertex>(), 0x20);
    }
}
