//! `marks::draw_particle` (0x4CFDD5): one particle of the marks' ring drawn as a square
//! sprite. Its place (st(0) x, st(1) y, st(2) z) turned by `g_viewMatrix` read by columns
//! into `flt_8EB8D5` .. `flt_8EB8DD`; unless the turned depth is at or below
//! `dword_8EB961` (a signed compare of the bits): the four vertices' colour ecx and depth
//! (times `flt_8EB96D`), half the square (edx times `flt_624088` / depth) in
//! `dword_8EB8B5` and `dword_8EB8B9`, the corners around the projected place (centre
//! `flt_62409C`, `flt_6240A0`), and `scene::draw_fan`. The vertices' rhw and texture places
//! are left as they are (`marks::set_quad_uvs` sets the places once a frame).
//!
//! In:  st(0) .. st(2) the place (popped), ecx the colour, edx the size
//! Out: nothing (no caller reads a register it leaves but esi, which it keeps)
//!
//! Callers: the ring's update (code at 0x4D03E7, 0x4D04BE, 0x4D05AC).

use crate::d3d::vertices_store::VERTICES;
use crate::marks::frame_state::DRAW_WORK;
use crate::math::x87;
use crate::race::matrices_store::MATRICES;
use crate::scene::draw_store::DRAW;

/// The Rust version (see the header): `colour` (ecx), `size` (edx).
pub unsafe fn draw_particle(place: [[u8; 10]; 3], colour: u32, size: u32) {
    unsafe {
        let turned = x87::column_turn(&place, MATRICES.view_matrix[0].addr() as u32);
        for (n, v) in turned.into_iter().enumerate() {
            DRAW_WORK.turned[n].set_bits(v);
        }
        let [a, b, c] = turned;
        if (c as i32) <= DRAW_WORK.near.bits() as i32 {
            return;
        }
        let k = [
            DRAW.view_scale.bits(),
            DRAW.view_centre[0].bits(),
            DRAW.view_centre[1].bits(),
            DRAW_WORK.z_k.bits(),
        ];
        let sq = x87::particle_square(a, b, c, k, size as i32);
        for n in 0..4 {
            let v = &VERTICES.vertices[n];
            v.colour.set(colour);
            v.z.set_bits(sq[1]);
            v.x.set_bits(sq[6 + n]);
            v.y.set_bits(sq[2 + n]);
        }
        DRAW_WORK.half[0].set_bits(sq[0]);
        DRAW_WORK.half[1].set_bits(sq[0]);
        crate::marks::clip_quad::draw_fan();
    }
}
