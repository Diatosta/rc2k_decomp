//! `effects::set_quad_uvs` (0x4CFD6A): writing to the depth buffer off, and the four texture
//! places of the 0x20-byte set at `CornerSlopes.slopes[0].values[0]` + 0xC60 put into the four vertices'
//! texture places at `Buffers.vertices.vertices[0].u` (0x20 bytes apart: +0x18 of Direct3D vertices from
//! 0x9605A0: the u, v of a transformed-and-lit Direct3D vertex, 0x20 bytes each).
//!
//! One caller, `effects::run_particles` (the marks' frame).

/// The record of `g_marksCornerSlopes` whose eight values are the set (0xC60 bytes on).
const SET_RECORD: usize = 0xC60 / 0x24;

pub fn set_quad_uvs(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    effects: &crate::effects::owner::Effects,
) {
    {
        crate::scene::depth_test_off::depth_test_off();
        for n in 0..4 {
            let values = &effects.blocks.corner_slopes.slopes[SET_RECORD].values;
            let uv: [u32; 2] = [values[2 * n].bits(), values[2 * n + 1].bits()];
            buffer.vertices[n].u.set_bits(uv[0]);
            buffer.vertices[n].v.set_bits(uv[1]);
        }
    }
}
