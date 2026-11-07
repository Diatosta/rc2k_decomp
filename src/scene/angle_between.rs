//! `scene::angle_between` (0x450223): the direction from one place to another as an
//! angle of sixteen bits a turn.
//!
//! In:  eax, ebx = the first place, ecx, edx = the second
//! Out: eax = the angle; ecx, edx = the two differences; everything else preserved
//!
//! `fpatan` of (ecx - eax) over (edx - ebx), over pi, times `flt_5C5E40` (0.5) and `flt_5C5D98`
//! (65536), rounded (`fistp`), its low sixteen bits; the result is also left in `dword_62400C`.
//! The instructions themselves, as `fpatan`'s result is the processor's. Four callers.

use crate::math::x87;
use crate::race::owner_course::Course;
use crate::scene::owner::scene;

// ---- game data ----

// ---- Rust version ----

/// The angle of the two differences.
pub fn angle_of(course: &Course, first: u32, second: u32) -> u32 {
    {
        let angle = x87::angle_scaled_to_int(
            first as i32,
            second as i32,
            course.numbers.half.bits(),
            course.numbers.k65536.bits(),
        ) as u32
            & 0xFFFF;
        scene().drawing.scene.scratch.set_bits(angle);
        angle
    }
}
