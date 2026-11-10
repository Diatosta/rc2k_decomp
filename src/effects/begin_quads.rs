//! `effects::begin_quads` (0x4C4453): the render set up for the marks' quads and a quad's
//! four vertices made ready. `scene::set_texture` with `dword_5D1E10`,
//! `scene::depth_buffer_on`, `scene::depth_test_off`, render states 0x15 = 4, 0x13 = 5,
//! 0x14 = 6, 0x1C = 0, 0x1B = 1 (`scene::set_render_state`); then each of the four
//! `g_polygonVertices` gets +0xC `dword_91FFA6`, +0x10 -1, +0x14 0, and its texture place
//! from `dword_91FFAA` .. `dword_91FFB6` (u: AA, B2, B2, AA; v: AE, AE, B6, B6);
//! `g_polygonVertexCount` 4, `flt_8EB96D` 1 / `flt_6241C8`, `dword_91FFBA` 0.
//!
//! Caller: `effects::draw_frame`.

use crate::d3d::vertices_store::VERTICES;
use crate::session::read_bits;

/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// The Rust version (see the header): `scene::set_texture`, `scene::depth_buffer_on`,
/// `scene::depth_test_off`, the render states (`scene::set_render_state`).
pub fn begin_quads(effects: &crate::effects::owner::Effects, scene: &crate::scene::owner::Scene) {
    let frame = &effects.frame;
    {
        crate::scene::set_texture::set_texture(scene.textures.kinds.kind_first_slots[202].bits());
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::depth_test_off::depth_test_off();
        for (value, state) in STATES {
            crate::scene::set_render_state::set_render_state(value, state);
        }
        let colour: u32 = frame.draw_frame.colour.get();
        let [a, b, c, d]: [u32; 4] = read_bits(&frame.draw_frame.places);
        let uv = [(a, b), (c, b), (c, d), (a, d)];
        for (k, (u, v)) in uv.into_iter().enumerate() {
            let at = &VERTICES.vertices[k];
            at.rhw.set_bits(colour);
            at.colour.set(0xFFFF_FFFF);
            at.specular.set(0);
            at.u.set_bits(u);
            at.v.set_bits(v);
        }
        crate::scene::polygon_work::front_end()
            .polygon_vertex_count
            .set_bits(4u32);
        frame.draw_work.z_k.set_bits(crate::math::x87::reciprocal(
            crate::scene::polygon_work::front_end()
                .window
                .farthest
                .bits(),
        ));
        frame.draw_frame.cleared.set(0);
    }
}
