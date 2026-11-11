//! `d3d::draw_hud_batch` (0x45A869): draws the HUD's batch, an indexed list of triangles.
//!
//! In:  nothing: the HUD block's vertices and their count, and its indices (16 bits) and
//!      their count
//! The device's DrawIndexedPrimitive with a triangle list (4) of transformed and lit
//! vertices (3) and the flags 0x1C. One caller, `hud::flush_batch`.
//! Platform: the original always runs. In the standalone program the rasteriser draws
//! each triangle (`platform_3d::draw_triangles`) and the result is 0.

/// Draws the HUD's batch.
pub fn draw_hud_batch(hud: &crate::hud::owner::Hud) {
    // The vertices and indices are the HUD block's (`hud::store`); the original's two
    // pointers to them are not kept.
    let past = |what: &str| -> ! {
        crate::fatal::raise::end(what, crate::errors::BadInput("past the HUD's batch"))
    };
    crate::platform_3d::draw_triangles(
        hud.block.vertex_count.get(),
        hud.block.index_count.get(),
        |n| {
            let Some(v) = hud.block.vertices.get(n as usize) else {
                past("a vertex")
            };
            [
                v.x.bits(),
                v.y.bits(),
                v.z.bits(),
                v.rhw.bits(),
                v.colour.get(),
                v.specular.get(),
                v.u.bits(),
                v.v.bits(),
            ]
        },
        |k| match hud.block.indices.get(k) {
            Some(i) => i.get(),
            None => past("an index"),
        },
    );
}
