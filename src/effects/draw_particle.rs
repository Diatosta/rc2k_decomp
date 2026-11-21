//! `effects::draw_particle`: one particle of the effects' ring drawn as a square sprite. Its place
//! (x, y, z) turned by the view matrix read by columns into the draw work's place; unless the
//! turned depth is at or below the nearest depth (a signed compare of the bits): the four vertices'
//! colour and depth (times the depth scale), half the square (the size times the view scale /
//! depth), the corners around the projected place (centre the view's centre), and
//! `scene::draw_fan`. The vertices' rhw and texture places are left as they are
//! (`effects::set_quad_uvs` sets the places once a frame).

use crate::math::x87;

/// `colour` is the vertices' colour, `size` the half side.
pub fn draw_particle(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    view_matrix: &crate::scene::matrix::Matrix,
    place: [[u8; 10]; 3],
    colour: u32,
    size: u32,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) {
    let effects_frame = &effects.frame;
    let turned = x87::column_turn_by(&place, view_matrix);
    for (n, v) in turned.into_iter().enumerate() {
        effects_frame.draw_work.borrow_mut().turned[n] = v;
    }
    if x87::bits_at_or_below(turned[2], effects_frame.draw_work.borrow().near) {
        return;
    }
    let k = effects_frame
        .draw_work
        .borrow()
        .view_k(scene.scratch.front_end_polygon());
    let sq = x87::particle_square_floats(turned, k, size as i32);
    for n in 0..4 {
        let v = &buffer.vertices[n];
        v.colour.set(colour);
        v.z.set(sq[1]);
        v.x.set(sq[6 + n]);
        v.y.set(sq[2 + n]);
    }
    effects_frame.draw_work.borrow_mut().half[0] = sq[0];
    effects_frame.draw_work.borrow_mut().half[1] = sq[0];
    crate::effects::clip_quad::draw_fan(buffer, scene);
}
