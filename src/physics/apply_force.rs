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
use crate::physics::tables_store::ForceRecord;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::race::owner_shared::Shared;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

/// The Rust version (the exact one, for `physics::apply_forces`):
/// `rec` on the car block `car`.
pub fn apply_force(
    shared: &Shared,
    rec: &ForceRecord,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) {
    let work = &shared.physics_tables.push_record;
    for (k, w) in work[..3].iter().enumerate() {
        w.set_bits(rec.force[k].bits());
    }
    let dt = extended(shared.physics_tables.step_time.bits());
    scale_vector(work, dt);
    scene.drawing.scene.scratch.set(rounded(dt));
    // The work car's matrix (`g_crashMatrix`), whichever block it is: the original's.
    turn_by_car_matrix(shared, work);
    for (v, w) in car.velocity.iter().zip(work) {
        v.set_bits(x87::add(w.bits(), v.bits()));
    }
    let p = car.velocity.each_ref().map(|v| extended(v.bits()));
    let out = rotate_point(&car.car_inverse_matrix, p);
    for (l, e) in car.local_velocity.iter().zip(out) {
        l.set_bits(rounded(e));
    }
}
