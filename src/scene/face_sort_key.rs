//! `scene::face_sort_key` (0x450564): the depth a face is sorted by.
//!
//! Takes: the face, the number of its vertices, the point numbers of its vertices (read once by
//! `scene::face_faces_away`)
//!
//! The depths of the face's points (each point's record 0x1000 + (number & 0xFFF), its +0x18)
//! are added one by one, each sum rounded to a float; the sum times the entry
//! `g_polygonVertexCount` of the table `CourseLists.face_key_factors[0]` (0x100 floats the game fills: one over the
//! number, by the use) is rounded to a whole number (`fistp`). One caller,
//! `scene::list_object_face`. Refuses a count of 0 (the original goes through 2^32), a vertex
//! count past the table, no face and no records.

use crate::errors::{BadInput, Refused};
use crate::math::x87;

// ---- game data ----

const FACTORS: u32 = 0x100;

// ---- Rust version ----

pub fn face_sort_key(
    poly: &crate::scene::polygon_work::PolygonWork,
    part: &crate::workers::PartScratch,
    lists: &crate::race::course_lists_store::CourseLists,
    face: usize,
    count: u32,
    points: &[u32],
) -> Result<(), Refused> {
    let vertices: u32 = poly.polygon_vertex_count.get();
    if face == 0
        || !crate::scene::point_records::held(lists)
        || count == 0
        || count > 0x1000
        || vertices >= FACTORS
    {
        return Err(BadInput("no face, no records, or a count the key cannot be made for").into());
    }
    let mut sum = 0f32.to_bits();
    for &point in &points[..count as usize] {
        let record = crate::scene::point_records::of_object_point(point);
        sum = x87::add(crate::scene::point_records::field(part, record, 0x18)?, sum).to_bits();
    }
    let factor = lists.face_key_factors[vertices as usize].get();
    poly.face_sort_key.set(x87::mul_to_int(sum, factor) as u32);
    Ok(())
}
