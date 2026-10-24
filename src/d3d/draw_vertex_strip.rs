//! `d3d::draw_vertex_strip` (0x41ADE1): draws a triangle strip of the vertices at ebx.
//!
//! In:  eax = the vertices' number, ebx = the vertices (transformed and lit, 0x20 bytes each)
//! Out: eax = Direct3D's result (0: drawn); ecx, edx as Direct3D leaves them
//! The device's (`g_d3dDevice`, an `IDirect3DDevice2`) DrawPrimitive (+0x74) with a
//! triangle strip (5: `D3DPT_TRIANGLESTRIP`; the fan is 6), transformed and lit vertices (3) and the flags 0x1C (not clipped,
//! no lighting, no extents). Callers: `drive::tyre_turned_quad`, `drive::tyre_quad` (the
//! tyre change's gauge and the replay viewer's panel).
//! Platform: the original always runs, here the fan is only noted (`d3d::recorder`). In
//! the standalone program the rasteriser draws it (`platform_3d::draw_strip`) and the
//! result is 0 (before 2026-10-19 the standalone skipped it: the viewer's panel did not
//! show; then drew it as a fan, each quad's second triangle wrong). No self-test.

use crate::d3d::recorder::{Call, note};
use crate::d3d::{VERTEX_LEN, Vertex};

/// Direct3D takes at most this many vertices in one primitive.
const MOST: u32 = 0x400;

unsafe fn vertices(at: u32, count: u32) -> Vec<Vertex> {
    (0..count.min(MOST) as usize)
        .map(|n| unsafe { ((at as usize + n * VERTEX_LEN) as *const Vertex).read_unaligned() })
        .collect()
}

/// The strip of `count` vertices at `at` drawn: Direct3D's result, always 0 here.
pub unsafe fn draw_vertex_strip(at: u32, count: u32) -> u32 {
    let swallowed = note(|| Call::Fan(unsafe { vertices(at, count) }));
    if !swallowed {
        crate::platform_3d::draw_strip(&unsafe { vertices(at, count) });
    }
    0
}
