//! `physics::blend_wheel_load` (0x440F70): unless `flt_71A208` is below `flt_614C64`
//! (C0): a slot by the signs of 0x71A1FC, 0x71A200, 0x71A204 (4 + 2 + 1 for each below 0)
//! and `flt_71A03C[slot]` moved towards 1 by `flt_71A208` / `flt_5C5D60` held to 0 .. 1
//! (through `dword_62400C`): a + (1 - a) share. What the eight are is a reading at best.
//!
//! Returns: the slot (when reached)
//!
//! Caller: `sub_43B635` (0x43B99A), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87;
use crate::math::x87_run::{compare_full, toward_one};
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::session::Cell32;

/// The Rust core on the variables' cells `v`. The registers the original leaves are read by none of its callers (edx the slot).
pub fn blend_at(race: &Race, v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    let less = |a: u32, b: u32| compare_full(a, b) & 0x100 != 0;
    if less(get(0), race.shared.physics_tables.wheel_load_limit.bits()) {
        return;
    }
    let zero: u32 = race.course.numbers.zero.bits();
    let one: u32 = race.course.numbers.one.bits();
    let mut slot = 0;
    for (k, add) in [(1, 4), (2, 2), (3, 1)] {
        if less(get(k), zero) {
            slot += add;
        }
    }
    put(4, x87::div(get(0), race.course.numbers.sixty_four.bits()));
    if less(get(4), zero) {
        put(4, x87::loaded(zero));
    }
    if !less(get(4), one) {
        put(4, x87::loaded(one));
    }
    let slot = 5 + slot;
    put(slot, toward_one(get(slot), get(4)));
}

fn game<'a>(shared: &'a Shared, scene: &'a crate::scene::owner::Scene) -> Vec<&'a dyn Cell32> {
    vec![
        &shared.car.contact_depth as &dyn Cell32,
        &shared.car.contact_point[0],
        &shared.car.contact_point[1],
        &shared.car.contact_point[2],
        &scene.drawing.scene.scratch,
        &shared.car.impact_octant_marks[0],
        &shared.car.impact_octant_marks[1],
        &shared.car.impact_octant_marks[2],
        &shared.car.impact_octant_marks[3],
        &shared.car.impact_octant_marks[4],
        &shared.car.impact_octant_marks[5],
        &shared.car.impact_octant_marks[6],
        &shared.car.impact_octant_marks[7],
    ]
}

/// The Rust version (the exact one, for `physics::car_physics_step`); `r` as the
/// original leaves are read by none of its callers.
pub fn blend_wheel_load(race: &Race, scene: &crate::scene::owner::Scene) {
    blend_at(race, &game(&race.shared, scene));
}
