//! `physics::stall_when_still` (0x440B9D): the length of the motion kept by
//! `physics::keep_motion` (0x61409C .., `physics::vector_length`, rounded into
//! `scratch`); when not 0 (C3 clear), `stall_figure` = the motion (0x71A17C ..) dot it
//! (`physics::dot_product`) over the length twice. Then, unless the length's size is
//! below `stall_speed`, with `stall_figure` below `stall_share` and `no_auto_neutral` 0: gear 0
//! and both drive speeds 0. Probably the engine stalling when the car is
//! stopped or turned back on itself: the motion's dot product against its own length.
//!
//! Caller: `physics::car_physics_step` (0x43B896), every physics step.

use crate::math::x87::{abs_compare, compare_full, ext_div2};
use crate::physics::car_store::Car;
use crate::physics::dot_product::dot_product;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::rounded_value;

pub fn stall_when_still(car: &Car, tables: &PhysicsTables, scene: &crate::scene::owner::Scene) {
    let kept = &tables.kept_motion;
    // The length goes through the scene's scratch dword (a float's bits).
    let length = rounded_value(vector_length(kept));
    scene.drawing.scene.get().scratch.set(length.to_bits());
    if compare_full(length, ZERO) & 0x4000 == 0 {
        tables.stall_figure.set(ext_div2(
            dot_product(&car.local_velocity, kept),
            length,
            length,
        ));
    }
    if abs_compare(length, &tables.stall_speed) & 0x100 != 0 {
        return;
    }
    if compare_full(&tables.stall_figure, &tables.stall_share) & 0x100 == 0 {
        return;
    }
    if tables.no_auto_neutral.get() != 0 {
        return;
    }
    car.gear.set(0);
    car.drive_speed_a.set(0.0);
    car.drive_speed_b.set(0.0);
}
