//! `scene::draw_flat_quad`: a flat quad on the screen through the polygon pipeline: the
//! polygon-drawn flag 0; the four polygon records' vertex pointers (`quads`, 0x20
//! apart), texture places (`quad_uv`), the corners' x (left, right) and y (top, bottom) from
//! `quad_corners` as floats, the fixed depth; the entries -1, colour `quad_colour`'s low 24
//! bits, the entries' `light` 0; four vertices, the record pool, the list 0 .. 3; then
//! `scene::scale_texture_coordinates`, `scene::clip_polygon_to_screen` and, with three vertices
//! or more left, mode 6, both `polygon_sides` switches set (1, 0), the depth buffer off
//! (`scene::depth_buffer_on::depth_buffer`), `scene::draw_polygon`, the polygon-drawn flag 1,
//! the depth buffer on; the colour back to white. The quad's cells are in the race variables, set
//! by the caller.
//!
//! A refusal from the scale, the clip, the depth buffer calls or the draw comes after the writes.

use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner_course::CourseReads;

#[allow(
    clippy::too_many_arguments,
    reason = "the view's polygon work, the part's scratch and the owners"
)]
pub fn draw_flat_quad(
    poly: &crate::scene::polygon_work_store::PolygonWork,
    part: &crate::scene::scratch_store::PartScratch,
    face: &crate::race::face_work::FaceWork,
    course: CourseReads<'_>,
    reads: crate::scene::workers::DrawReads<'_>,
    screen: &crate::memory::session::Screen,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    let uv = view.quad.borrow().uv;
    for (n, rec) in part.polygon.entries.iter().enumerate().take(4) {
        rec.record
            .set(crate::scene::point_records::of_quad(n as u32));
        rec.u.set(uv[n]);
        rec.v.set(uv[4 + n]);
    }
    let float = |value: u32| x87::from_int(value as i32);
    let [left, right, top, bottom] = {
        let corners = view.quad.borrow().corners;
        [corners[0], corners[1], corners[4], corners[5]].map(float)
    };
    let v = |n: usize, o: usize, x: f32| {
        poly.quads.borrow_mut().words[8 * n + o / 4] = x.to_bits();
    };
    for (n, x, y) in [
        (0, left, top),
        (1, right, top),
        (2, right, bottom),
        (3, left, bottom),
    ] {
        v(n, 0xC, x);
        v(n, 0x1C, y);
    }
    let depth = course.numbers.k4096_b.get();
    for n in 0..4 {
        v(n, 0x18, depth);
    }
    for item in part.polygon.entries.iter().take(4) {
        item.point.set(0xFFFF_FFFF);
    }
    poly.scalars.borrow_mut().polygon_colour = view.quad.borrow().colour[0] & 0xFF_FFFF;
    for item in part.polygon.entries.iter().take(4) {
        item.light.set(0);
    }
    poly.scalars.borrow_mut().polygon_vertex_count = 4;
    poly.scalars.borrow_mut().polygon_next_record = crate::scene::polygon::pool_start();
    for n in 0..4u32 {
        crate::scene::polygon::list_cell(&part.polygon, n as usize).set(n);
    }
    crate::scene::polygon::list_cell(&part.polygon, 4).set(0);
    poly.scalars.borrow_mut().polygon_next_entry = 4;
    crate::scene::scale_texture_coordinates::scale_texture_coordinates(&part.polygon, poly)?;
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen(part, poly, course)?;
    if poly.scalars.borrow().polygon_vertex_count >= 3 {
        face.polygon_mode.set(6);
        poly.scalars.borrow_mut().polygon_sides[0] = 1;
        poly.scalars.borrow_mut().polygon_sides[1] = 0;
        crate::scene::depth_buffer_on::depth_buffer(part, false);
        crate::scene::draw_polygon::draw_polygon_in_game(view, part, reads, screen)?;
        crate::scene::depth_buffer_on::depth_buffer(part, true);
    }
    poly.scalars.borrow_mut().polygon_colour = 0xFF_FFFF;
    Ok(())
}
