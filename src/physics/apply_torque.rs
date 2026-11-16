//! Applies one force record (the force and the point it acts at) as a torque on a car block. The
//! spin is turned into the car's frame; each part is reduced by the matching part of the cross
//! product of the record's two vectors times the physics step times the car's inverse inertia;
//! the result is turned back into the world and stored as the spin.

use crate::math::x87::neg_product2_plus;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::tables_store::ForceRecord;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::turn_by_car_matrix::turn_cells;
use crate::scene::view_matrix::cross_product;

/// The exact version, for `physics::apply_torques`: the force record at `rec` (a record of the
/// wheels' or the contacts' table) on the car block `car`.
pub fn apply_torque(tables: &PhysicsTables, contact: &ContactWork, rec: &ForceRecord, car: &Car) {
    let work = &contact.contact_turn_speed;
    let cross = &tables.torque_cross;
    for (w, s) in work.iter().zip(&car.spin) {
        w.set(s.get());
    }
    turn_cells(&car.car_inverse_matrix, work);
    cross_product(&rec.force, &rec.point, cross);
    let step = &tables.step_time;
    for k in 0..3 {
        work[k].set(neg_product2_plus(
            &cross[k],
            step,
            &tables.inertia[k],
            &work[k],
        ));
    }
    turn_cells(&car.crash_matrix, work);
    for (s, w) in car.spin.iter().zip(work) {
        s.set(w.get());
    }
}
