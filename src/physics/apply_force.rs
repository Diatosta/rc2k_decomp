//! `physics::apply_force` (0x45FAE1): applies one force record (its first three floats) to a
//! car block. The force is copied to `push_record`, multiplied by the physics step
//! (`physics::scale_vector`), turned into the world by `g_crashMatrix`
//! (`physics::turn_by_car_matrix`) and added to the car's motion; the motion is then turned by
//! the car's inverse matrix (`scene::rotate_point`) into the car-frame speed (the last of which is
//! `g_carSpeed`). A reading: a force's push on the car.
//!
//! Takes: the record, the car block
//!
//! Caller: `physics::apply_forces` (in a loop over records).

use crate::math::x87;
use crate::physics::car_store::Car;
use crate::physics::scale_vector::scale_vector;
use crate::physics::tables_store::ForceRecord;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

/// The exact version, for `physics::apply_forces`: `rec` on the car block `car`.
pub fn apply_force(
    car: &Car,
    tables: &PhysicsTables,
    rec: &ForceRecord,
    scene: &crate::scene::owner::Scene,
) {
    let work = &tables.push_record;
    for (k, w) in work[..3].iter().enumerate() {
        w.set_bits(rec.force[k].bits());
    }
    let dt = extended(tables.step_time.bits());
    scale_vector(work, dt);
    scene.drawing.scene.scratch.set(rounded(dt));
    // The work car's matrix (`g_crashMatrix`), whichever block it is: the original's.
    turn_by_car_matrix(car, work);
    for (v, w) in car.velocity.iter().zip(work) {
        v.set(x87::add(w, v));
    }
    let p = car.velocity.each_ref().map(|v| extended(v.get()));
    let out = rotate_point(&car.car_inverse_matrix, p);
    for (l, e) in car.local_velocity.iter().zip(out) {
        l.set_bits(rounded(e));
    }
}
