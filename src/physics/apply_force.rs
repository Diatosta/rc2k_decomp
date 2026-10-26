//! `physics::apply_force` (0x45FAE1): one force record (esi, its first three floats) on a
//! car (edi, its block): copied to `flt_614B30` .., times the step `flt_61388C`
//! (`physics::scale_vector`; the step, left in st(0), then rounded into `dword_62400C`),
//! turned into the world by `g_crashMatrix` (`physics::turn_by_car_matrix`), added to the
//! car's motion (+0x198 .. +0x1A0); then the motion turned by the car's inverse matrix
//! (+0x9CC, `scene::rotate_point`) into +0x1A4 .. +0x1AC (the last `g_carSpeed`). All in
//! `pusha` / `popa`. A reading: a force's push on the car.
//!
//! In:  esi the record, edi the car block
//! Out: every register kept; no flags read
//!
//! Caller: `physics::apply_forces` (in a loop over records).

use crate::math::x87;
use crate::physics::car_store::Car;
use crate::physics::scale_vector::scale_vector;
use crate::physics::tables_store::PHYSICS;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

unsafe fn at(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::apply_force's operand")
}

unsafe fn put(a: u32, v: u32) {
    crate::pointers::write_or_end::<u32>(a, v, "physics::apply_force's operand")
}

/// The Rust core: the record at `rec`, the car at `car`, the work vector at `work`, the
/// scratch at `scratch`.
pub unsafe fn apply_at(rec: u32, car: u32, work: u32, scratch: u32) {
    unsafe {
        for k in 0..3 {
            put(work + 4 * k, at(rec + 4 * k));
        }
        let dt = extended(PHYSICS.step_time.bits());
        scale_vector(work, dt);
        put(scratch, rounded(dt));
        turn_by_car_matrix(work);
        for k in 0..3 {
            let o = car + 0x198 + 4 * k;
            put(o, x87::add(at(work + 4 * k), at(o)));
        }
        let p = [
            extended(Car::at(car).velocity[0].bits()),
            extended(Car::at(car).velocity[1].bits()),
            extended(Car::at(car).velocity[2].bits()),
        ];
        let out = rotate_point(car + 0x9CC, p);
        for (k, e) in out.iter().enumerate() {
            put(car + 0x1A4 + 4 * k as u32, rounded(*e));
        }
    }
}

/// The Rust version (the exact one, for `physics::apply_forces`).
pub unsafe fn apply_force(rec: u32, car: u32) {
    unsafe {
        apply_at(
            rec,
            car,
            PHYSICS.push_record[0].addr() as u32,
            DRAW.scratch.addr() as u32,
        )
    }
}
