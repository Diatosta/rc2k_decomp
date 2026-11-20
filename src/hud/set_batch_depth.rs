//! The four polygon vertices given the HUD's rhw (`HudBlock.rhw`), depth (`HudBlock.depth`) and
//! specular 0.

use crate::d3d::vertices_store::PolygonVertices;
use crate::hud::owner::Hud;

use crate::math::x87;

pub(super) fn set_batch_depth(hud: &Hud, buffer: &PolygonVertices) {
    {
        let rhw = x87::loaded(&hud.block.rhw);
        let depth = x87::loaded(&hud.block.depth);
        for item in buffer.vertices.iter().take(4) {
            let v = &item;
            v.rhw.set(rhw);
            v.specular.set(0);
            v.z.set(depth);
        }
    }
}
