//! `scene::face_sort_key` (0x450564): the depth a face is sorted by.
//!
//! In:  ebp = the face, ecx = the number of its vertices
//! Out: `dword_62407C` = the key; eax = the last point's record as an offset, ebx = the
//!      number of vertices as `g_polygonVertexCount` has it, ecx = 0, edx = the last
//!      point's record, esi = the face's end; everything else preserved
//!
//! The depths of the face's points (each point's record 0x1000 + (number & 0xFFF), its +0x18)
//! are added one by one, each sum rounded to a float; the sum times the entry
//! `g_polygonVertexCount` of the table `flt_5C6950` (0x100 floats the game fills: one over the
//! number, by the use) is rounded to a whole number (`fistp`). One caller,
//! `scene::list_object_face`. Refuses a count of 0 (the original goes through 2^32), a vertex
//! count past the table, no face and no records.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::scene::owner::scene;

// ---- game data ----

const FACTORS: u32 = 0x100;

// ---- Rust version ----

pub fn face_sort_key(
    course: &crate::race::owner_course::Course,
    face: usize,
    count: u32,
) -> Result<(), Refused> {
    let drawing = &scene().drawing;
    let vertices: u32 = drawing.scene.polygon_vertex_count.bits();
    if face == 0
        || !crate::scene::point_records::held()
        || count == 0
        || count > 0x1000
        || vertices >= FACTORS
    {
        return Err(BadInput("no face, no records, or a count the key cannot be made for").into());
    }
    let mut sum = 0f32.to_bits();
    for n in 0..count as usize {
        let record = crate::scene::point_records::of_object_point(
            crate::scene::object_faces::word((face + 0x10 + n * 0x10) as u32)?,
        );
        sum = x87::add(crate::scene::point_records::field(record, 0x18)?, sum);
    }
    let factor = course.lists.face_key_factors[vertices as usize].bits();
    drawing
        .scene
        .face_sort_key
        .set_bits(x87::mul_to_int(sum, factor) as u32);
    Ok(())
}
