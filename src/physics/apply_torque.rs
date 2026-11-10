//! `physics::apply_torque` (0x45FB82): one force record (esi: the force at +0, the point
//! it acts at at +0xC) turning a car (edi, its block): the car's spin (+0x184 .. +0x18C)
//! copied to `dword_6D30B4` .. and turned into the car's frame
//! (`physics::turn_by_inverse_at`, 0x45FC55: `turn_cells` on the block's inverse matrix); the cross product of the record's two vectors
//! (`scene::cross_product`, into `flt_614274` ..); each part of the spin less that part
//! times the step `flt_61388C` times `flt_613FA8 + 4k` (`fld c; fmul; fmul; fchs; fadd;
//! fstp`); turned back into the world (`physics::turn_by_matrix_at`, 0x45FC73: on its +0xB4) and stored as the
//! spin. All in `pusha` / `popa` (eax pushed besides). A reading: the force's twist on
//! the car, `flt_613FA8` the inverse inertia.
//!
//! Takes: the record, the car block
//!
//! Caller: `physics::apply_torques` (in a loop over records).

use crate::math::x87_run::neg_product2_plus;
use crate::physics::car_store::Car;
use crate::physics::tables_store::ForceRecord;
use crate::physics::turn_by_car_matrix::turn_cells;
use crate::race::owner::Race;
use crate::scene::view_matrix::cross_product;

/// The exact version, for `physics::apply_torques`: the force record at `rec` (a record of the
/// wheels' or the contacts' table) on the car block `car`.
pub fn apply_torque(race: &Race, rec: &ForceRecord, car: &Car) {
    let work = &race.physics.contact.contact_turn_speed;
    let cross = &race.shared.physics_tables.torque_cross;
    for (w, s) in work.iter().zip(&car.spin) {
        w.set_bits(s.bits());
    }
    turn_cells(&car.car_inverse_matrix, work);
    cross_product(&rec.force, &rec.point, cross);
    let step = race.shared.physics_tables.step_time.bits();
    for k in 0..3 {
        let inertia = race.shared.physics_tables.inertia[k].bits();
        work[k].set_bits(neg_product2_plus(
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
