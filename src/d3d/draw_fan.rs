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
use crate::framework::Regs;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's). In the
/// hooked game it goes to the game's function, whose hook notes the call for the tests'
/// recorder (`d3d::recorder`) and runs the original: noted here as well, a call would be
/// noted twice.
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::platform_3d::draw_fan(&polygon());
    r.eax = 0;
    Ok(())
}
