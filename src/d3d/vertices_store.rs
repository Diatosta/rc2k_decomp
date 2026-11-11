//! The polygon vertex buffer (IDA 0x9605A0 .. 0x960DA0): 64 transformed and lit vertices (`TLVertex`, 0x20 bytes) in the image's zeroed tail
//! (`race::buffers_store`). The polygon about to be drawn stands in its first vertices
//! (`DRAW.polygon_vertex_count` says how many).

use crate::race::owner::race;
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

/// The piece, a field of `crate::race::owner::race().shared.buffers` for the game's own thread
/// and one of `WORKER_COPIES` for each other drawing thread.
pub struct InBuffers;

/// The drawing threads' copies but the game's thread's (`workers`).
static WORKER_COPIES: [PolygonVertices; crate::workers::COUNT - 1] =
    [const { crate::pointers::cells_of_bytes([0u8; core::mem::size_of::<PolygonVertices>()]) };
        crate::workers::COUNT - 1];

impl core::ops::Deref for InBuffers {
    type Target = PolygonVertices;
    #[inline]
    fn deref(&self) -> &PolygonVertices {
        crate::workers::pick(&race().shared.buffers.vertices, &WORKER_COPIES)
    }
}

impl crate::workers::Copies for InBuffers {
    fn copy(&self, n: usize) -> &dyn crate::pointers::Store {
        match n {
            0 => &race().shared.buffers.vertices,
            n => &WORKER_COPIES[n - 1],
        }
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
