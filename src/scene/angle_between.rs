//! `scene::angle_between` (0x450223): the direction from one place to another as an
//! angle of sixteen bits a turn.
//!
//! In:  eax, ebx = the first place, ecx, edx = the second
//! Out: eax = the angle; ecx, edx = the two differences; everything else preserved
//!
//! `fpatan` of (ecx - eax) over (edx - ebx), over pi, times `flt_5C5E40` (0.5) and `flt_5C5D98`
//! (65536), rounded (`fistp`), its low sixteen bits; the result is also left in `dword_62400C`.
//! The instructions themselves, as `fpatan`'s result is the processor's. Four callers.

use crate::framework::{game_read, game_write, rva};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;

// ---- game data ----

const HALF_RVA: usize = rva(0x5C5E40); // flt_5C5E40, 0.5
const TURN_RVA: usize = rva(0x5C5D98); // flt_5C5D98, 65536.0

// ---- Rust version ----

/// The angle of the two differences.
pub unsafe fn angle_of(first: u32, second: u32) -> u32 {
    unsafe {
        let angle = x87::angle_scaled_to_int(
            first as i32,
            second as i32,
            game_read(HALF_RVA),
            game_read(TURN_RVA),
        ) as u32
            & 0xFFFF;
        game_write(CHECKED_RVA, angle);
        angle
    }
}
