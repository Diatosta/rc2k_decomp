//! `effects::begin_quads`: the render set up for the effects' quads and a quad's four vertices made
//! ready: `scene::set_texture` with the effects' texture, `scene::depth_buffer_on`,
//! `scene::depth_test_off`, render states 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 0, 0x1B = 1
//! (`scene::set_render_state`); then each of the four polygon vertices gets the quad's `rhw`,
//! the colour -1, the specular 0, and its texture place from the quad's constants (u: a, b, b, a;
//! v: c, c, d, d); the polygon's vertex count 4, the depth scale 1 / the farthest depth, the
//! quad's `cleared` flag 0.

use crate::memory::session::PlainFloat;

/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// Sets the texture, the depth state and the render states, then readies the four vertices.
pub(super) fn begin_quads(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    effects: &crate::effects::owner::Effects,
    scene: &crate::scene::owner::Scene,
) {
    let part = scene.scratch.game_scratch();
    let frame = &effects.frame;
    {
        crate::scene::set_texture::set_texture(
            part,
            scene,
            scene.textures.kinds.get().kind_first_slots[202].get(),
        );
        crate::scene::depth_buffer_on::depth_buffer(part, true);
        crate::scene::depth_test_off::depth_test_off(part);
        for (value, state) in STATES {
            crate::scene::set_render_state::set_render_state(part, value, state);
        }
        let rhw = frame.draw_frame.rhw.get();
        let [a, b, c, d] = frame.draw_frame.places.each_ref().map(PlainFloat::get);
        let uv = [(a, b), (c, b), (c, d), (a, d)];
        for (k, (u, v)) in uv.into_iter().enumerate() {
            let at = &buffer.vertices[k];
            at.rhw.set(rhw);
            at.colour.set(0xFFFF_FFFF);
            at.specular.set(0);
            at.u.set(u);
            at.v.set(v);
        }
        scene
            .scratch
            .front_end_polygon()
            .polygon_vertex_count
            .set(4);
        frame.draw_work.borrow_mut().z_k =
            crate::math::x87::reciprocal(&scene.scratch.front_end_polygon().window.farthest);
        frame.draw_frame.cleared.set(0);
    }
}
