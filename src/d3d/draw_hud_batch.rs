//! `d3d::draw_hud_batch`: draws the HUD's batch, an indexed list of triangles of transformed and
//! lit vertices. The HUD block's vertices and indices (16 bits) and their counts are the input. The
//! rasteriser draws each triangle (`d3d::device::draw_triangles`); the result is 0.

/// Draws the HUD's batch.
pub fn draw_hud_batch(
    part: &crate::scene::scratch_store::PartScratch,
    hud: &crate::hud::owner::Hud,
) {
    // The vertices and indices are the HUD block's (`hud::block_store`).
    let past = |what: &str| -> ! {
        crate::fatal::raise::end(what, crate::errors::BadInput("past the HUD's batch"))
    };
    crate::d3d::device::draw_triangles(
        part,
        hud.block.vertex_count,
        hud.block.index_count,
        |n| {
            let Some(v) = hud.block.vertices.get(usize::from(n)) else {
                past("a vertex")
            };
            crate::d3d::wire_vertex([v.x, v.y, v.z, v.rhw], v.colour, v.specular, [v.u, v.v])
        },
        |k| match hud.block.indices.get(k) {
            Some(&i) => i,
            None => past("an index"),
        },
    );
}
