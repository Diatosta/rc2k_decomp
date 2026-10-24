//! `d3d::draw_fan` (0x41AD7F): draws the polygon in the vertex buffer, clipped by the
//! device.
//!
//! In:  nothing: the vertices are at `g_polygonVertices` (0x9605A0), their number in
//!      `g_polygonVertexCount` (0x6240F8)
//! Out: eax = Direct3D's result (0: drawn); ecx, edx as Direct3D leaves them
//! The device's DrawPrimitive with a triangle fan (6) of transformed and lit vertices
//! (3) and no flags (clipped). One caller, `scene::draw_fan`.
//! Platform: the original always runs, here the fan is only noted (`d3d::recorder`). In
//! the standalone program the rasteriser draws it (`platform_3d::draw_fan`, held to the
//! screen) and the result is 0. No self-test.

use crate::d3d::draw_fan_unclipped::polygon;

/// The fan drawn: Direct3D's result, always 0 here.
pub fn draw_fan() -> u32 {
    crate::platform_3d::draw_fan(&polygon());
    0
}
