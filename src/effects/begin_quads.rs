//! `effects::begin_quads` (0x4C4453): the render set up for the effects' quads and a quad's
//! four vertices made ready. `scene::set_texture` with the effects' texture,
//! `scene::depth_buffer_on`, `scene::depth_test_off`, render states 0x15 = 4, 0x13 = 5,
//! 0x14 = 6, 0x1C = 0, 0x1B = 1 (`scene::set_render_state`); then each of the four polygon
//! vertices gets +0xC the quad's value, +0x10 -1, +0x14 0, and its texture place from the
//! quad's constants (u: a, b, b, a; v: c, c, d, d); the polygon's vertex count 4, the depth
//! scale 1 / the farthest depth, the quad's last cell 0.//!
//! Caller: `effects::draw_frame`.

use crate::session::read_bits;

/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// `scene::set_texture`, `scene::depth_buffer_on`,
/// `scene::depth_test_off`, the render states (`scene::set_render_state`).
pub fn begin_quads(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    effects: &crate::effects::owner::Effects,
    scene: &crate::scene::owner::Scene,
) {
    let part = crate::workers::game_scratch();
    let frame = &effects.frame;
    {
        crate::scene::set_texture::set_texture(
            part,
            scene,
            scene.textures.kinds.kind_first_slots[202].get(),
        );
        crate::scene::depth_buffer_on::depth_buffer(part, true);
        crate::scene::depth_test_off::depth_test_off(part);
        for (value, state) in STATES {
            crate::scene::set_render_state::set_render_state(part, value, state);
        }
        let colour: u32 = frame.draw_frame.colour.get();
        let [a, b, c, d]: [u32; 4] = read_bits(&frame.draw_frame.places);
        let uv = [(a, b), (c, b), (c, d), (a, d)];
        for (k, (u, v)) in uv.into_iter().enumerate() {
            let at = &buffer.vertices[k];
            at.rhw.set_bits(colour);
            at.colour.set(0xFFFF_FFFF);
            at.specular.set(0);
            at.u.set_bits(u);
            at.v.set_bits(v);
        }
        crate::scene::polygon_work::front_end()
            .polygon_vertex_count
            .set_bits(4u32);
        frame.draw_work.z_k.set(crate::math::x87::reciprocal(
            &crate::scene::polygon_work::front_end().window.farthest,
        ));
        frame.draw_frame.cleared.set(0);
    }
}
