//! `physics::turn_by_heading` (0x43F136): the x and z floats of a point turned by minus the work
//! car's heading (`car_heading`) with `math::turn_float_point_xz` (y passed through) and stored
//! back.
//!
//! Takes: the point, passed on
//!
//! Caller: `physics::point_hits_car`.

use crate::math::turn_float_point_xz::turn_float_point_xz;
use crate::physics::car_store::Car;

/// `point` is the three floats (x, y, z); x and z are turned.
pub fn turn_by_heading(car: &Car, point: &[crate::session::Float; 3], ebx: u32) {
    let angle = car.car_heading.get().wrapping_neg();
    let [x, _, z, _] = turn_float_point_xz(point[0].bits(), ebx, point[2].bits(), angle);
    point[0].set_bits(x);
    point[2].set_bits(z);
}
