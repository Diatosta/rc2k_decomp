//! With all four wheels down (the four `wheel_squeeze` dwords not 0), the spin's x and z are held
//! to -`ground_spin_limit` .. `ground_spin_limit`, and `ground_spin` = whether either now has the
//! limit's size (also when no number). A reading: no tumbling on the ground.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use rc2k_core::physics::motion::hold_spin;

/// Compares and copies of floats only: the same at any precision.
// Core: `rc2k_core::physics::motion::hold_spin`.
pub(super) fn hold_spin_on_ground(car: &Car, tables: &PhysicsTables) {
    // Wheels down: all four squeeze words (integers in float cells, tested as words) are not 0.
    if car.wheel_squeeze.iter().any(|w| w.bits() == 0) {
        return;
    }
    let (mut limit, mut x, mut z) = (
        tables.ground_spin_limit.get(),
        car.spin[0].get(),
        car.spin[2].get(),
    );
    let flag = hold_spin(&mut limit, &mut x, &mut z);
    tables.ground_spin_limit.set(limit);
    car.spin[0].set(x);
    car.spin[2].set(z);
    tables.ground_spin.set(flag != 0);
}
