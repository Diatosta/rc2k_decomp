//! `scene::allocate_point_records` (0x41F976): takes the memory of the points'
//! records.
//!
//! In:  nothing
//! Out: eax = the memory, ebx = the list it is taken on; everything else preserved
//!
//! 0x40000 bytes (0x2000 records of 0x20 bytes: 0x1000 for a scene's points and 0x1000 for an
//! object's) on the list `stru_5C6E50`, counted in `dword_5C6F90`, to `g_pointRecords`. Two
//! callers, the front end's 3D set-up and the stage's loading.

use crate::errors::Refused;
use crate::race::owner_course::Course;
use crate::scene::point_records::{self, BYTES};

// ---- Rust version ----

/// The records taken (Rust's since phase 13, `scene::point_records`): counted on the
/// list, `g_pointRecords` set to `point_records::HELD`.
pub fn allocate_point_records_in_game(course: &Course) -> Result<(), Refused> {
    course
        .lists
        .course_bytes_total
        .set_bits(course.lists.course_bytes_total.bits().wrapping_add(BYTES));
    point_records::take(course, &course.lists.point_records_list)
}
