//! `marks::view_motion` (0x4C4D40): the view's motion since the last frame. With the
//! view matrix `flt_5EA3CC`: its third row (+0x20 ..) kept on the stack; its place (the
//! origin turned by it, `scene::rotate_point`, unrounded) less the last (`flt_91FFBE` ..)
//! into `flt_8EB951` .. and the stack; that step made length 1
//! (`scene::normalise_vector3`); `flt_8F397D` the third row's dot product with it; the
//! place (turned again) kept as the last; then `flt_8EB951` .. each over
//! `g_marksFrameTicks` (`fidiv`).
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4C3EEE`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::marks::frame_state::{DRAW_FRAME, DRAW_WORK, VIEW_VARS};
use crate::marks::setup::CLOCK;
use crate::math::x87;
use crate::race::matrices_store::MATRICES;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::{dot_product3, normalise_vector3};

/// The Rust version (see the header).
pub unsafe fn view_motion() {
    unsafe {
        let m = MATRICES.view_place[0].addr() as u32;
        let get = |a: u32| (a as *const u32).read_unaligned();
        let mut local = [0u32; 6];
        for (k, v) in local.iter_mut().take(3).enumerate() {
            *v = x87::loaded(get(m + 0x20 + 4 * k as u32));
        }
        let zero = [extended(0); 3];
        let place = rotate_point(m, zero);
        for k in 0..3 {
            let step = x87::ext_less(place[k], DRAW_FRAME.last[k].bits());
            DRAW_WORK.step[k].set_bits(step);
            local[3 + k] = step;
        }
        let at = local.as_mut_ptr() as u32;
        normalise_vector3(at + 12);
        VIEW_VARS
            .facing
            .set_bits(rounded(dot_product3(at, at + 12)));
        let place = rotate_point(m, zero);
        for (k, p) in place.iter().enumerate() {
            DRAW_FRAME.last[k].set_bits(rounded(*p));
        }
        let ticks: i32 = CLOCK.frame_ticks.get() as i32;
        for k in 0..3 {
            let a = &DRAW_WORK.step[k];
            a.set_bits(x87::over_int(a.bits(), ticks));
        }
    }
}
