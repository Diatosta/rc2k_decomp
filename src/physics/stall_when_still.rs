//! The length of the motion kept by `physics::keep_motion` (`physics::vector_length`, rounded); when
//! not 0, `stall_figure` = the motion dot it (`physics::dot_product`) over the
//! length twice. Then, unless the length's size is below `stall_speed`, with `stall_figure` below
//! `stall_share` and `no_auto_neutral` 0: gear 0 and both drive speeds 0. Probably the engine
//! stalling when the car is stopped or turned back on itself: the motion's dot product against its
//! own length.

use crate::math::matrix3::rounded_value;
use crate::math::x87::{abs_compare, compare_full, ext_div2};
use crate::physics::car_store::Car;
use crate::physics::dot_product::dot_product;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::vector_length::vector_length;

pub fn stall_when_still(car: &Car, tables: &PhysicsTables) {
    let kept = &tables.kept_motion;
    let length = rounded_value(vector_length(kept));
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
