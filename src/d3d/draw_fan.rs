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
use crate::d3d::recorder::{Call, note};
use crate::framework::{RawHook, Regs, hook_asm, rva, standalone};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("d3d::draw_fan", rva(0x41AD7F), thunk, &ORIGINAL);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let swallowed = note(|| Call::Fan(polygon()));
    if swallowed {
        r.eax = 0;
        return;
    }
    if standalone() {
        #[cfg(feature = "standalone")]
        crate::platform_3d::draw_fan(&polygon());
        r.eax = 0;
        return;
    }
    unsafe { call_original(r) };
}
