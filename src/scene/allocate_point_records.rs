//! `scene::allocate_point_records`: takes the memory of the points' records.
//!
//! 0x40000 bytes (0x2000 records of 0x20 bytes: 0x1000 for a scene's points and 0x1000 for an
//! object's) on the list `CourseLists.point_records_list[0]`, counted in
//! `CourseLists.course_bytes_total`, and kept as the point records.

use crate::errors::Refused;
use crate::race::owner_course::Course;
use crate::scene::point_records::{self, BYTES};

/// The records are Rust's (`scene::point_records`): counted on the list, the point records set
/// to `point_records::HELD`.
pub fn allocate_point_records_in_game(
    scratch: &crate::scene::scratch_store::DrawingScratch,
    course: &Course,
) -> Result<(), Refused> {
    {
        let mut lists = course.lists.borrow_mut();
        lists.course_bytes_total = lists.course_bytes_total.wrapping_add(BYTES);
    }
    point_records::take(scratch, course, &course.memory().point_records_list)
}
