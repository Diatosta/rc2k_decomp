//! `physics::stall_when_still` (0x440B9D): the length of the motion kept by
//! `physics::keep_motion` (0x61409C .., `physics::vector_length`, rounded into
//! `dword_62400C`); when not 0 (C3 clear), `flt_6140AC` = the motion (0x71A17C ..) dot it
//! (`physics::dot_product`) over the length twice. Then, unless the length's size is
//! below `flt_613E40`, with `flt_6140AC` below `flt_613E44` and `dword_61416C` 0: gear 0
//! and both drive speeds 0. By the look of it the engine stalling when the car turns
//! back on itself (a reading).
//!
//! In:  nothing
//! Out: esi 0x71A17C and edi 0x61409C when the dot was taken, else esi 0x61409C; the rest
//!      kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B896), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87_run::{abs_compare, compare_full, ext_div2};
use crate::physics::car_store::CAR;
use crate::physics::dot_product::dot_product;
use crate::physics::tables_store::PHYSICS;
use crate::physics::vector_length::vector_length;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::rounded;
use crate::session::Cell32;

/// The Rust core on the variables' cells `v`. The registers the original leaves are read by none of its callers (esi, edi).
pub fn stall_at(v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    put(3, rounded(vector_length(&v[0..3])));
    let zero: u32 = NUMBERS.zero.bits();
    if compare_full(get(3), zero) & 0x4000 == 0 {
        put(7, ext_div2(dot_product(&v[4..7], &v[0..3]), get(3), get(3)));
    }
    if abs_compare(get(3), PHYSICS.stall_speed.bits()) & 0x100 != 0 {
        return;
    }
    if compare_full(get(7), PHYSICS.stall_share.bits()) & 0x100 == 0 {
        return;
    }
    if get(8) != 0 {
        return;
    }
    put(9, 0);
    put(10, 0);
    put(11, 0);
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &PHYSICS.kept_motion[0] as &dyn Cell32,
        &PHYSICS.kept_motion[1],
        &PHYSICS.kept_motion[2],
        &DRAW.scratch,
        &CAR.local_velocity[0],
        &CAR.local_velocity[1],
        &CAR.local_velocity[2],
        &PHYSICS.stall_figure,
        &PHYSICS.no_auto_neutral,
        &CAR.gear,
        &CAR.drive_speed_a,
        &CAR.drive_speed_b,
    ]
}

/// The Rust version.
pub fn stall_when_still() {
    stall_at(&game())
}
