//! `marks::squares_setup` (0x4BFEAC): the squares' frame set up: `scene::depth_buffer_on`,
//! `scene::set_texture` (`dword_5D1E10`), render states 0x11 = 2, 0x12 = 2, 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 0, 0x1B = 1; four vertices of `g_polygonVertices` with rhw 0x100
//! (through `dword_62400C`), colour -1, specular 0xFF000000, texture places from
//! `dword_8BC212` .. (u a, c, c, a; v b, b, d, d); `g_polygonVertexCount` 4; `flt_89CEEA` 1
//! over `flt_6241C8`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BFFEC` (0x4C000F).

use crate::d3d::vertices_store::VERTICES;
use crate::marks::squares_store::{SQUARE_STATE, SQUARE_TABLES};
use crate::math::x87;
use crate::scene::draw_store::DRAW;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::session::read_bits;
const STATES: [(u32, u32); 7] = [
    (2, 0x11),
    (2, 0x12),
    (4, 0x15),
    (5, 0x13),
    (6, 0x14),
    (0, 0x1C),
    (1, 0x1B),
];

/// The Rust version (see the header).
pub fn squares_setup() {
    setup_with(read_bits(&SQUARE_STATE.setup_places))
}

/// The set-up with the texture places `places` (`marks::pong_setup` has its own).
pub fn setup_with(places: [u32; 4]) {
    use crate::scene::set_render_state::set_render_state;
    {
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::set_texture::set_texture(KIND_TEXTURES.kind_first_slots[202].bits());
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        for n in 0..4 {
            DRAW.scratch.set_bits(0x100u32);
            VERTICES.vertices[n].rhw.set_bits(x87::int_to_float(0x100));
        }
        for n in 0..4 {
            VERTICES.vertices[n].colour.set(0xFFFF_FFFFu32);
            VERTICES.vertices[n].specular.set(0xFF00_0000u32);
        }
        let [a, b, c, d] = places;
        for (n, (u, v)) in [(a, b), (c, b), (c, d), (a, d)].into_iter().enumerate() {
            VERTICES.vertices[n].u.set_bits(u);
            VERTICES.vertices[n].v.set_bits(v);
        }
        DRAW.polygon_vertex_count.set_bits(4u32);
        SQUARE_TABLES
            .inverse
            .set_bits(x87::reciprocal(DRAW.farthest.bits()));
    }
}
