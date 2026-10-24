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

use crate::framework::rva;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;

const VIEW_RVA: usize = rva(0x5EA2CC); // g_viewMatrix
const TURNED_RVA: Slot = Slot::at(0x8EB8D5); // flt_8EB8D5 .. flt_8EB8DD
const NEAR_RVA: Slot = Slot::at(0x8EB961); // dword_8EB961
const HALF_RVA: Slot = Slot::at(0x8EB8B5); // dword_8EB8B5, dword_8EB8B9
const FOCAL_RVA: usize = rva(0x624088); // flt_624088
const CENTRE_RVA: [usize; 2] = [rva(0x62409C), rva(0x6240A0)];
const Z_K_RVA: Slot = Slot::at(0x8EB96D); // flt_8EB96D
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices: TLVertex[4]

/// The Rust version (see the header): `colour` (ecx), `size` (edx).
pub unsafe fn draw_particle(place: [[u8; 10]; 3], colour: u32, size: u32) {
    unsafe {
        let turned = x87::column_turn(&place, game_addr(VIEW_RVA) as u32);
        for (n, v) in turned.into_iter().enumerate() {
            game_write(TURNED_RVA + 4 * n, v);
        }
        let [a, b, c] = turned;
        if (c as i32) <= game_read::<i32>(NEAR_RVA) {
            return;
        }
        let k = [
            game_read(FOCAL_RVA),
            game_read(CENTRE_RVA[0]),
            game_read(CENTRE_RVA[1]),
            game_read(Z_K_RVA),
        ];
        let sq = x87::particle_square(a, b, c, k, size as i32);
        for n in 0..4 {
            let v = VERTICES_RVA + 0x20 * n;
            game_write(v + 0x10, colour);
            game_write(v + 8, sq[1]);
            game_write(v, sq[6 + n]);
            game_write(v + 4, sq[2 + n]);
        }
        game_write(HALF_RVA, sq[0]);
        game_write(HALF_RVA + 4, sq[0]);
        crate::marks::clip_quad::draw_fan();
    }
}
