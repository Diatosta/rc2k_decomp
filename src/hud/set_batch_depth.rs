//! `hud::set_batch_depth` (0x4B726C): the four vertices of `g_polygonVertices` given the HUD's rhw
//! (`HudBlock.rhw`), depth (`HudBlock.depth`) and specular 0.
//!
//! Caller: `hud::draw_hud` (the HUD's start).

use crate::d3d::vertices_store::PolygonVertices;
use crate::hud::owner::Hud;

use crate::math::x87;

pub fn set_batch_depth(hud: &Hud, buffer: &PolygonVertices) {
    {
        let rhw = x87::loaded(&hud.block.rhw);
        let depth = x87::loaded(&hud.block.depth);
        for n in 0..4 {
            let v = &buffer.vertices[n];
            v.rhw.set(rhw);
            v.specular.set(0);
            v.z.set(depth);
        }
    }
}
