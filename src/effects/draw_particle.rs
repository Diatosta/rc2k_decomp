//! `effects::draw_particle` (0x4CFDD5): one particle of the marks' ring drawn as a square
//! sprite. Its place (st(0) x, st(1) y, st(2) z) turned by `g_viewMatrix` read by columns
//! into `flt_8EB8D5` .. `flt_8EB8DD`; unless the turned depth is at or below
//! `dword_8EB961` (a signed compare of the bits): the four vertices' colour ecx and depth
//! (times `flt_8EB96D`), half the square (edx times `flt_624088` / depth) in
//! `dword_8EB8B5` and `dword_8EB8B9`, the corners around the projected place (centre
//! `flt_62409C`, `flt_6240A0`), and `scene::draw_fan`. The vertices' rhw and texture places
//! are left as they are (`effects::set_quad_uvs` sets the places once a frame).
//!
//! Callers: the ring's update (code at 0x4D03E7, 0x4D04BE, 0x4D05AC).

use crate::d3d::vertices_store::VERTICES;
use crate::math::x87;
use crate::race::owner_frame::Frame;

/// The Rust version (see the header): `colour` (ecx), `size` (edx).
pub fn draw_particle(
    frame: &Frame,
    place: [[u8; 10]; 3],
    colour: u32,
    size: u32,
    effects: &crate::effects::owner::Effects,
    scene: &crate::scene::owner::Scene,
) {
    let effects_frame = &effects.frame;
    let turned = x87::column_turn(&place, &frame.matrices.view_matrix);
    for (n, v) in turned.into_iter().enumerate() {
        effects_frame.draw_work.turned[n].set_bits(v);
    }
    let [a, b, c] = turned;
    if (c as i32) <= effects_frame.draw_work.near.bits() as i32 {
        return;
    }
    let k = [
        scene.drawing.scene.view_scale.bits(),
        scene.drawing.scene.view_centre[0].bits(),
        scene.drawing.scene.view_centre[1].bits(),
        effects_frame.draw_work.z_k.bits(),
    ];
    let sq = x87::particle_square(a, b, c, k, size as i32);
    for n in 0..4 {
        let v = &VERTICES.vertices[n];
        v.colour.set(colour);
        v.z.set_bits(sq[1]);
        v.x.set_bits(sq[6 + n]);
        v.y.set_bits(sq[2 + n]);
    }
    effects_frame.draw_work.half[0].set_bits(sq[0]);
    effects_frame.draw_work.half[1].set_bits(sq[0]);
    crate::effects::clip_quad::draw_fan();
}
