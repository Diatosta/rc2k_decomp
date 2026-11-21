//! `scene::face_sort_key`: the depth a face is sorted by.
//!
//! Takes the face, the number of its vertices and the point numbers of its vertices (read once by
//! `scene::face_faces_away`). The depths of the face's points (each point's record 0x1000 + (number
//! & 0xFFF), its depth) are added one by one, each sum rounded to a float; the sum times the entry
//! `polygon_vertex_count` of the table `CourseLists.face_key_factors` (0x100 floats the game fills:
//! one over the number, by the use) is rounded to a whole number. Refuses a count of 0, a vertex
//! count past the table, no face and no records.

use crate::errors::{BadInput, Refused};
use crate::math::x87;

// ---- game data ----

const FACTORS: u32 = 0x100;

pub fn face_sort_key(
    poly: &crate::scene::polygon_work_store::PolygonWork,
    part: &crate::scene::scratch_store::PartScratch,
    lists: crate::race::course_lists_store::CourseListsRef<'_>,
    face: usize,
    count: u32,
    points: &[u32],
) -> Result<(), Refused> {
    let vertices: u32 = poly.scalars.borrow().polygon_vertex_count;
    if face == 0
        || !crate::scene::point_records::held(lists)
        || count == 0
        || count > 0x1000
        || vertices >= FACTORS
    {
        return Err(BadInput("no face, no records, or a count the key cannot be made for").into());
    }
    let mut sum = 0f32.to_bits();
    crate::math::x87_float::at_rounding!(|r| {
        for &point in &points[..count as usize] {
            let record = crate::scene::point_records::of_object_point(point);
            let depth = crate::scene::point_records::field(part, record, 0x18)?;
            sum = x87::add_with(r, depth, sum).to_bits();
        }
    });
    let factor = lists.face_key_factor(vertices as usize);
    poly.scalars.borrow_mut().face_sort_key = x87::mul_to_int(sum, factor) as u32;
    Ok(())
}
