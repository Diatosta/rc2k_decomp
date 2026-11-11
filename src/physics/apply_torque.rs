//! `physics::apply_torque` (0x45FB82): one force record (the force, and the point it acts at)
//! turning a car block. The car's spin is copied to `contact_turn_speed` and turned into the
//! car's frame (`physics::turn_by_inverse_at`); the cross product of the record's two vectors
//! is taken (`scene::cross_product`, into `torque_cross`); each part of the spin is reduced by
//! that part of the cross product times the physics step times the car's inverse inertia, and the
//! result is turned back into the world (`physics::turn_by_matrix_at`) and stored as the spin.
//! A reading: the force's twist on the car.
//!
//! Takes: the record, the car block
//!
//! Caller: `physics::apply_torques` (in a loop over records).

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
        w.set_bits(s.bits());
    }
    turn_cells(&car.car_inverse_matrix, work);
    cross_product(&rec.force, &rec.point, cross);
    let step = tables.step_time.bits();
    for k in 0..3 {
        let inertia = tables.inertia[k].bits();
        work[k].set(neg_product2_plus(
            cross[k].bits(),
            step,
            inertia,
            work[k].bits(),
        ));
    }
    turn_cells(&car.crash_matrix, work);
    for (s, w) in car.spin.iter().zip(work) {
        s.set_bits(w.bits());
    }
}
