//! With `contact_flag` set, in gear, the engine speed below `stall_engine_speed` (also when
//! unordered) and `no_auto_neutral` 0: gear 0, `drive_cut_steps` from the tables,
//! `gear_change_hold` = 0x14, `gear_changed` and `gear_change_pending` 1 (as the gear changes of
//! `physics::change_gear` set them). Probably the engine stalling into neutral.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use rc2k_core::physics::gearing::drops_to_neutral;

/// A compare of floats only: the same at any precision.
// Core: `rc2k_core::physics::gearing::drops_to_neutral`.
pub(super) fn drop_to_neutral(car: &Car, tables: &PhysicsTables) {
    if !drops_to_neutral(
        car.contact_flag.get(),
        car.gear.get(),
        car.engine_speed.get(),
        tables.stall_engine_speed.get(),
        || tables.no_auto_neutral.get(),
    ) {
        return;
    }
    car.gear.set(0);
    car.drive_cut_steps.set(tables.drive_cut_steps.get());
    car.gear_change_hold.set(0x14);
    car.gear_changed.set(1);
    car.gear_change_pending.set(1);
}
