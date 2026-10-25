//! `hud::set_batch_depth` (0x4B726C): the four vertices of `g_polygonVertices` given the
//! HUD's rhw (`flt_75BFEC`), depth (`flt_75BFE8`) and specular 0.
//!
//! In:  nothing
//! Out: all kept
//!
//! Caller: `sub_4B1CD0` (the HUD's start).

use crate::d3d::vertices_store::VERTICES;

use crate::hud::store::HUD;
use crate::math::x87;

/// The Rust version (see the header).
pub unsafe fn set_batch_depth() {
    {
        let rhw = x87::loaded(HUD.rhw.bits());
        let depth = x87::loaded(HUD.depth.bits());
        for n in 0..4 {
            let v = &VERTICES.vertices[n];
            v.rhw.set_bits(rhw);
            v.specular.set(0);
            v.z.set_bits(depth);
        }
    }
}
