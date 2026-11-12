//! `physics::air_drag` (0x43F2DA): for the motion's three (`local_velocity`,
//! `Car.local_velocity[1]`, `g_carSpeed`): its square times `air_drag` over `k10000` into
//! `PhysicsTables.forces[5].force[0]` .. `PhysicsTables.forces[5].force[2]`, made against the motion (|it| when the motion is below 0
//! or no number, else -|it|). This is the air's drag: a force against the motion that grows with the square of the speed.
//!
//! Caller: `physics::car_physics_step` (0x43B80E), every physics step.

use crate::math::x87;
use crate::math::x87::{abs_of, compare_full, square_times_over};
use crate::physics::car_store::Car;
use crate::physics::flow::{FloatView, ZERO_BITS as ZERO};
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
/// The drag of this step against each of the car's three motions.
pub fn air_drag(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    let (k, d, zero) = (&tables.air_drag, &numbers.k10000, ZERO);
    for i in 0..3 {
        let (motion, force) = (&car.local_velocity[i], &tables.forces[5].force[i]);
        force.set_float(square_times_over(motion.get(), k, d));
        let a = abs_of(force.float());
        let below = compare_full(motion.get(), zero) & 0x100 != 0;
        force.set_float(if below { a } else { x87::negated(a) });
    }
}
