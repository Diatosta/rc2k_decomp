//! `effects::draw_particle` (0x4CFDD5): one particle of the effects' ring drawn as a square
//! sprite. Its place (st(0) x, st(1) y, st(2) z) turned by the view matrix read by columns
//! into the draw work's place; unless the turned depth is at or below the nearest depth (a
//! signed compare of the bits): the four vertices' colour and depth (times the depth scale),
//! half the square (the size times the view scale / depth), the corners around the projected
//! place (centre the view's centre), and `scene::draw_fan`. The vertices' rhw and texture
//! places are left as they are (`effects::set_quad_uvs` sets the places once a frame).
//!
//! Callers: the ring's update.

use crate::d3d::vertices_store::VERTICES;
use crate::math::x87;

/// `colour` is the vertices' colour, `size` the half side.
pub fn draw_particle(
    view_matrix: &[crate::session::Float; 16],
    place: [[u8; 10]; 3],
    colour: u32,
    size: u32,
    effects: &crate::effects::owner::Effects,
) {
    let effects_frame = &effects.frame;
    let turned = x87::column_turn(&place, view_matrix);
    for (n, v) in turned.into_iter().enumerate() {
        effects_frame.draw_work.turned[n].set(v);
    }
    let [a, b, c] = turned;
    if (c.to_bits() as i32) <= effects_frame.draw_work.near.bits() as i32 {
        return;
    }
    let k = [
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .bits(),
        crate::scene::polygon_work::front_end().window.view_centre[0].bits(),
        crate::scene::polygon_work::front_end().window.view_centre[1].bits(),
        effects_frame.draw_work.z_k.bits(),
    ];
    let sq = x87::particle_square(a, b, c, k, size as i32);
    for n in 0..4 {
        let v = &VERTICES.vertices[n];
        v.colour.set(colour);
        v.z.set(sq[1]);
        v.x.set(sq[6 + n]);
        v.y.set(sq[2 + n]);
    }
    effects_frame.draw_work.half[0].set(sq[0]);
    effects_frame.draw_work.half[1].set(sq[0]);
    crate::effects::clip_quad::draw_fan();
}
