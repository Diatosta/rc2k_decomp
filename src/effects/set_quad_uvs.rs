//! `effects::set_quad_uvs`: writing to the depth buffer off, and the four texture places of the
//! 0x20-byte set at `CornerSlopes.slopes[0].values[0]` + 0xC60 put into the four vertices' texture
//! places at `Buffers.vertices.vertices[0].u` (the u, v of a transformed-and-lit Direct3D vertex;
//! the vertices are 0x20 bytes apart).

/// The record of the corners' slopes whose eight values are the set (0xC60 bytes on).
const SET_RECORD: usize = 0xC60 / 0x24;

pub fn set_quad_uvs(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) {
    {
        crate::scene::depth_test_off::depth_test_off(scene.scratch.game_scratch());
        for n in 0..4 {
            let values = &effects.blocks.corner_slopes.slopes[SET_RECORD].values;
            buffer.vertices[n].u.set(values[2 * n].get());
            buffer.vertices[n].v.set(values[2 * n + 1].get());
        }
    }
}
