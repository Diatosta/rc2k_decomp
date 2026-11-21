//! Applies one force record (the force and the point it acts at) as a torque on a car block. The
//! spin is turned into the car's frame; each part is reduced by the matching part of the cross
//! product of the record's two vectors times the physics step times the car's inverse inertia;
//! the result is turned back into the world and stored as the spin.

use crate::math::x87::neg_product2_plus;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::tables_store::ForceRecord;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::vector_bits::{bits_of, turn};
use crate::scene::view_matrix::cross_product;
use std::cell::RefCell;

/// The exact version, for `physics::apply_torques`: the force record at `rec` (a record of the
/// wheels' or the contacts' table) on the car block `car`.
pub fn apply_torque(
    tables: &PhysicsTables,
    contact: &RefCell<ContactWork>,
    rec: &ForceRecord,
    car: &Car,
) {
    let cross = &tables.torque_cross;
    let mut work = turn(&car.car_inverse_matrix, bits_of(&car.spin));
    cross_product(&rec.force, &rec.point, cross);
    let step = tables.step_time.get();
    for k in 0..3 {
        work[k] = neg_product2_plus(&cross[k], step, &tables.inertia[k], work[k]).to_bits();
    }
    work = turn(&car.crash_matrix, work);
    for (s, w) in car.spin.iter().zip(work) {
        s.set_bits(w);
    }
    // The turn speed is the step's work place: it ends as the car's new spin.
    contact.borrow_mut().contact_turn_speed = work;
}
