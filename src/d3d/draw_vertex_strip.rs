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
//! show; then drew it as a fan, each quad's second triangle wrong).

use crate::d3d::recorder::{Call, note};
use crate::d3d::{VERTEX_LEN, Vertex};
use crate::errors::BadInput;
use crate::pointers::GamePtr;

/// Direct3D takes at most this many vertices in one primitive.
const MOST: u32 = 0x400;

fn vertices(at: GamePtr, count: u32) -> Result<Vec<Vertex>, BadInput> {
    let bytes = at.read_bytes(count.min(MOST) as usize * VERTEX_LEN, "a strip's vertices")?;
    Ok(bytes
        .as_chunks::<VERTEX_LEN>()
        .0
        .iter()
        .map(|vertex| {
            std::array::from_fn(|n| {
                u32::from_le_bytes(vertex[4 * n..4 * n + 4].try_into().unwrap())
            })
        })
        .collect())
}

/// The strip of `count` vertices at `at` drawn: Direct3D's result, always 0 here.
pub fn draw_vertex_strip(at: GamePtr, count: u32) -> Result<u32, BadInput> {
    let vertices = vertices(at, count)?;
    let swallowed = note(|| Call::Fan(vertices.clone()));
    if !swallowed {
        crate::platform_3d::draw_strip(&vertices);
    }
    Ok(0)
}
