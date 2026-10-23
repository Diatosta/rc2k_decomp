//! `d3d::draw_fan_unclipped` (0x41AD9C): draws the polygon in the vertex buffer.
//!
//! In:  nothing: the vertices are at `g_polygonVertices` (0x9605A0), their number in
//!      `g_polygonVertexCount` (0x6240F8)
//! Out: eax = Direct3D's result (0: drawn); ecx, edx as Direct3D leaves them
//! The device's DrawPrimitive with a triangle fan (6) of transformed and lit vertices
//! (3) and the flags 0x1C (not to be clipped, lit or measured). Two callers,
//! `scene::draw_polygon_fan` and `sub_4651E6`.
//! Platform: the original always runs, here the fan is only noted. In the standalone
//! program the rasteriser draws it (`platform_3d::draw_fan`) and the result is 0. No
//! self-test.

use crate::d3d::{VERTEX_LEN, Vertex};
use crate::framework::{Regs, game_addr, game_read, rva};

pub const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
pub const COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
/// More than the buffer can hold is not noted: 0x6240F8 lies well past 0x9605A0 only by
/// address, and a count gone wrong must not be read through.
const MOST: u32 = 0x100;

/// The polygon as it stands in the buffer.
pub fn polygon() -> Vec<Vertex> {
    let count = unsafe { game_read::<u32>(COUNT_RVA) }.min(MOST) as usize;
    let at = game_addr(VERTICES_RVA);
    (0..count)
        .map(|n| unsafe { ((at + n * VERTEX_LEN) as *const Vertex).read_unaligned() })
        .collect()
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's). In the
/// hooked game it goes to the game's function, whose hook notes the call for the tests'
/// recorder (`d3d::recorder`) and runs the original: noted here as well, a call would be
/// noted twice.
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::platform_3d::draw_fan(&polygon());
    r.eax = 0;
    Ok(())
}
