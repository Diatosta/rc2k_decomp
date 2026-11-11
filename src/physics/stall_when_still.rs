//! `physics::stall_when_still` (0x440B9D): the length of the motion kept by
//! `physics::keep_motion` (0x61409C .., `physics::vector_length`, rounded into
//! `dword_62400C`); when not 0 (C3 clear), `flt_6140AC` = the motion (0x71A17C ..) dot it
//! (`physics::dot_product`) over the length twice. Then, unless the length's size is
//! below `flt_613E40`, with `flt_6140AC` below `flt_613E44` and `dword_61416C` 0: gear 0
//! and both drive speeds 0. Probably the engine stalling when the car is
//! stopped or turned back on itself: the motion's dot product against its own length.
//!
//! Caller: `physics::car_physics_step` (0x43B896), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87_run::{abs_compare, compare_full, ext_div2};
use crate::physics::car_store::Car;
use crate::physics::dot_product::dot_product;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::rounded;
use crate::session::Cell32;

/// On the variables' cells `v`.
pub fn stall_at(tables: &PhysicsTables, v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    put(3, rounded(vector_length(&v[0..3])));
    let zero: u32 = ZERO;
    if compare_full(get(3), zero) & 0x4000 == 0 {
        put(7, ext_div2(dot_product(&v[4..7], &v[0..3]), get(3), get(3)));
    }
    if abs_compare(get(3), tables.stall_speed.bits()) & 0x100 != 0 {
        return;
    }
    if compare_full(get(7), tables.stall_share.bits()) & 0x100 == 0 {
        return;
    }
    if get(8) != 0 {
        return;
    }
    put(9, 0);
    put(10, 0);
    put(11, 0);
}

fn game<'a>(
    car: &'a Car,
    tables: &'a PhysicsTables,
    scene: &'a crate::scene::owner::Scene,
) -> Vec<&'a dyn Cell32> {
    vec![
        &tables.kept_motion[0] as &dyn Cell32,
        &tables.kept_motion[1],
        &tables.kept_motion[2],
        &scene.drawing.scene.scratch,
        &car.local_velocity[0],
        &car.local_velocity[1],
        &car.local_velocity[2],
        &tables.stall_figure,
        &tables.no_auto_neutral,
        &car.gear,
        &car.drive_speed_a,
        &car.drive_speed_b,
    ]
}

pub fn stall_when_still(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    stall_at(tables, &game(car, tables, scene));
}
