//! `scene::angle_between`: the direction from one place to another as an angle of sixteen
//! bits a turn.
//!
//! `fpatan` of the first difference over the second, over pi, times 0.5 and 65536, rounded, its low
//! sixteen bits. It runs the instructions themselves, as `fpatan`'s result is the processor's.

use crate::math::x87;
use crate::race::owner_course::Course;

// ---- game data ----

/// The angle of the two differences.
pub fn angle_of(course: &Course, first: u32, second: u32) -> u32 {
    x87::angle_scaled_to_int(
        first as i32,
        second as i32,
        course.numbers.get().half.get(),
        course.numbers.get().k65536.get(),
    ) as u32
        & 0xFFFF
}
