//! `effects::windscreen_flakes_setup`: the windscreen flakes' frame set up:
//! `scene::depth_buffer_on`, `scene::set_texture` (the flakes' texture), render states 0x11 = 2,
//! 0x12 = 2, 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 0, 0x1B = 1; four polygon vertices with rhw
//! 0x100, colour -1, specular 0xFF000000, texture places from the flakes' constants (u a, c, c, a;
//! v b, b, d, d); the polygon's vertex count 4; the flakes' `inverse` 1 over the farthest depth.//!

use crate::effects::windscreen_flakes_store::WindscreenFlakes;

use crate::math::x87;
const STATES: [(u32, u32); 7] = [
    (2, 0x11),
    (2, 0x12),
    (4, 0x15),
    (5, 0x13),
    (6, 0x14),
    (0, 0x1C),
    (1, 0x1B),
];

pub(super) fn windscreen_flakes_setup(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    fl: &WindscreenFlakes,
    scene: &crate::scene::owner::Scene,
) {
    setup_with(buffer, fl, fl.state.borrow().setup_places, scene);
}

/// The set-up with the texture places `places` (`effects::pong_setup` has its own).
pub(super) fn setup_with(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    fl: &WindscreenFlakes,
    places: [f32; 4],
    scene: &crate::scene::owner::Scene,
) {
    use crate::scene::set_render_state::set_render_state;
    {
        crate::scene::depth_buffer_on::depth_buffer(scene.scratch.game_scratch(), true);
        crate::scene::set_texture::set_texture(
            scene.scratch.game_scratch(),
            scene,
            scene.textures.kinds.borrow().kind_first_slots[202],
        );
        for (value, state) in STATES {
            set_render_state(scene.scratch.game_scratch(), value, state);
        }
        for item in buffer.vertices.iter().take(4) {
            item.rhw.set(x87::from_int(0x100));
        }
        for item in buffer.vertices.iter().take(4) {
            item.colour.set(0xFFFF_FFFFu32);
            item.specular.set(0xFF00_0000u32);
        }
        let [a, b, c, d] = places;
        for (n, (u, v)) in [(a, b), (c, b), (c, d), (a, d)].into_iter().enumerate() {
            buffer.vertices[n].u.set(u);
            buffer.vertices[n].v.set(v);
        }
        scene
            .scratch
            .front_end_polygon()
            .polygon_vertex_count
            .set(4);
        fl.tables.borrow_mut().inverse =
            x87::reciprocal(scene.scratch.front_end_polygon().window.borrow().farthest);
    }
}
