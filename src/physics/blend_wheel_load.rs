//! Unless `contact_depth` is below `wheel_load_limit`: a slot chosen by the signs of three
//! impact values (4 + 2 + 1 for each below 0) has its `impact_octant_marks` entry moved towards 1
//! by `contact_depth` / `sixty_four` held to 0 .. 1 (a + (1 - a) share). What the eight slots
//! mean is a reading at best.

use crate::math::x87;
use crate::math::x87::{compare_full, toward_one};
use crate::physics::car_store::Car;
use crate::physics::figures::WHEEL_LOAD_LIMIT;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::race::numbers_store::Numbers;
/// The blend of this step.
pub(super) fn blend_wheel_load(car: &Car, numbers: &Numbers) {
    let depth = &car.contact_depth;
    let less = |a: f32, b: f32| compare_full(a, b) & 0x100 != 0;
    if less(depth.get(), WHEEL_LOAD_LIMIT) {
        return;
    }
    let zero = f32::from_bits(ZERO);
    let one = numbers.one.get();
    let mut slot = 0;
    for (k, add) in [(0, 4), (1, 2), (2, 1)] {
        if less(car.contact_point[k].get(), zero) {
            slot += add;
        }
    }
    let mut share = x87::div(depth.get(), &numbers.sixty_four);
    if less(share, zero) {
        share = x87::loaded(zero);
    }
    if !less(share, one) {
        share = x87::loaded(one);
    }
    let mark = &car.impact_octant_marks[slot];
    mark.set(toward_one(mark.get(), share));
}
