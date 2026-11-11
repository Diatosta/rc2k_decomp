//! `physics::blend_wheel_load` (0x440F70): unless `contact_depth` is below `wheel_load_limit`
//! (C0): a slot by the signs of 0x71A1FC, 0x71A200, 0x71A204 (4 + 2 + 1 for each below 0)
//! and `impact_octant_marks`[slot]` moved towards 1 by `contact_depth` / `sixty_four` held to 0 .. 1
//! (through `scratch`): a + (1 - a) share. What the eight are is a reading at best.
//!
//! Returns: the slot (when reached)
//!
//! Caller: `physics::car_physics_step` (0x43B99A), every physics step.

use crate::math::x87;
use crate::math::x87::{compare_full, toward_one};
use crate::physics::car_store::Car;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::session::Cell32;

/// On the variables' cells `v`.
pub fn blend_at(tables: &PhysicsTables, numbers: &Numbers, v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    let less = |a: u32, b: u32| compare_full(a, b) & 0x100 != 0;
    if less(get(0), tables.wheel_load_limit.bits()) {
        return;
    }
    let zero: u32 = ZERO;
    let one: u32 = numbers.one.bits();
    let mut slot = 0;
    for (k, add) in [(1, 4), (2, 2), (3, 1)] {
        if less(get(k), zero) {
            slot += add;
        }
    }
    put(4, x87::div(get(0), &numbers.sixty_four).to_bits());
    if less(get(4), zero) {
        put(4, x87::loaded(zero).to_bits());
    }
    if !less(get(4), one) {
        put(4, x87::loaded(one).to_bits());
    }
    let slot = 5 + slot;
    put(slot, toward_one(get(slot), get(4)).to_bits());
}

fn game<'a>(car: &'a Car, scene: &'a crate::scene::owner::Scene) -> Vec<&'a dyn Cell32> {
    vec![
        &car.contact_depth as &dyn Cell32,
        &car.contact_point[0],
        &car.contact_point[1],
        &car.contact_point[2],
        &scene.drawing.scene.scratch,
        &car.impact_octant_marks[0],
        &car.impact_octant_marks[1],
        &car.impact_octant_marks[2],
        &car.impact_octant_marks[3],
        &car.impact_octant_marks[4],
        &car.impact_octant_marks[5],
        &car.impact_octant_marks[6],
        &car.impact_octant_marks[7],
    ]
}

/// The exact version, for `physics::car_physics_step`.
pub fn blend_wheel_load(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    scene: &crate::scene::owner::Scene,
) {
    blend_at(tables, numbers, &game(car, scene));
}
