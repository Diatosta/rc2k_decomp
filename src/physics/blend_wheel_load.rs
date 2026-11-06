//! `physics::blend_wheel_load` (0x440F70): unless `flt_71A208` is below `flt_614C64`
//! (C0): a slot by the signs of 0x71A1FC, 0x71A200, 0x71A204 (4 + 2 + 1 for each below 0)
//! and `flt_71A03C[slot]` moved towards 1 by `flt_71A208` / `flt_5C5D60` held to 0 .. 1
//! (through `dword_62400C`): a + (1 - a) share. What the eight are is a reading at best.
//!
//! In:  nothing
//! Out: edx the slot (when reached); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B99A), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87;
use crate::math::x87_run::{compare_full, toward_one};
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner_course::Course;
use crate::session::Cell32;

/// The Rust core on the variables' cells `v`. The registers the original leaves are read by none of its callers (edx the slot).
pub fn blend_at(course: &Course, v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    let less = |a: u32, b: u32| compare_full(a, b) & 0x100 != 0;
    if less(get(0), PHYSICS.wheel_load_limit.bits()) {
        return;
    }
    let zero: u32 = course.numbers.zero.bits();
    let one: u32 = course.numbers.one.bits();
    let mut slot = 0;
    for (k, add) in [(1, 4), (2, 2), (3, 1)] {
        if less(get(k), zero) {
            slot += add;
        }
    }
    put(4, x87::div(get(0), course.numbers.sixty_four.bits()));
    if less(get(4), zero) {
        put(4, x87::loaded(zero));
    }
    if !less(get(4), one) {
        put(4, x87::loaded(one));
    }
    let slot = 5 + slot;
    put(slot, toward_one(get(slot), get(4)));
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &CAR.contact_depth as &dyn Cell32,
        &CAR.contact_point[0],
        &CAR.contact_point[1],
        &CAR.contact_point[2],
        &crate::scene::owner::scene().drawing.scene.scratch,
        &CAR.impact_octant_marks[0],
        &CAR.impact_octant_marks[1],
        &CAR.impact_octant_marks[2],
        &CAR.impact_octant_marks[3],
        &CAR.impact_octant_marks[4],
        &CAR.impact_octant_marks[5],
        &CAR.impact_octant_marks[6],
        &CAR.impact_octant_marks[7],
    ]
}

/// The Rust version (the exact one, for `physics::car_physics_step`); `r` as the
/// original leaves are read by none of its callers.
pub fn blend_wheel_load(course: &Course) {
    blend_at(course, &game());
}
