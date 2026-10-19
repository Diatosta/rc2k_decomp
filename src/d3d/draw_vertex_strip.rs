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
use crate::framework::{RawHook, Regs, hook_asm, rva, standalone};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("d3d::draw_vertex_strip", rva(0x41ADE1), thunk, &ORIGINAL);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Direct3D takes at most this many vertices in one primitive.
const MOST: u32 = 0x400;

fn vertices(r: &Regs) -> Vec<Vertex> {
    (0..r.eax.min(MOST) as usize)
        .map(|n| unsafe { ((r.ebx as usize + n * VERTEX_LEN) as *const Vertex).read_unaligned() })
        .collect()
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let swallowed = note(|| Call::Fan(vertices(r)));
    if swallowed {
        r.eax = 0;
        return;
    }
    if standalone() {
        #[cfg(feature = "standalone")]
        crate::platform_3d::draw_strip(&vertices(r));
        r.eax = 0;
        return;
    }
    unsafe { call_original(r) };
}
