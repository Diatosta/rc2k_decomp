//! Applies one force record (its first three floats) to a car block. The force is scaled by the
//! physics step, turned into the world by the crash matrix and added to the car's motion; the
//! motion is then turned by the car's inverse matrix into the car-frame speed.

use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;
use crate::physics::flow::FloatView;
use crate::physics::scale_vector::scale_vector;
use crate::physics::tables_store::ForceRecord;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::race::numbers_store::Numbers;
use crate::scene::rotate_point::rotate_point;

/// The exact version, for `physics::apply_forces`: `rec` on the car block `car`.
pub fn apply_force(car: &Car, numbers: &Numbers, rec: &ForceRecord) {
    // The push in the car's frame, worked out on the spot (the fourth dword stays 0).
    let work = [const { PlainDword::new(0) }; 4];
    for (k, w) in work[..3].iter().enumerate() {
        w.set_float(rec.force[k].get());
    }
    let dt = extended(numbers.step_time.get());
    scale_vector(&work, dt);
    // The work car's crash matrix, whichever block it is.
    turn_by_car_matrix(car, &work);
    for (v, w) in car.velocity.iter().zip(work) {
        v.set(x87::add(w, v));
    }
    let p = car.velocity.each_ref().map(|v| extended(v.get()));
    let out = rotate_point(&car.car_inverse_matrix, p);
    for (l, e) in car.local_velocity.iter().zip(out) {
        l.set(rounded_value(e));
    }
}
