//! `physics::drop_to_neutral` (0x43FD71): with `dword_71A20C` set, in gear, the engine
//! speed below `flt_6140FC` (`fcomp`, C0: also when unordered) and `dword_61416C` 0: gear
//! 0, `dword_71A1A8` = `dword_613FA4`, `dword_71A1A4` = 0x14, `dword_71A210` and
//! `dword_71A214` 1 (as the gear changes of `physics::change_gear` set them). Probably the engine
//! stalling into neutral: the engine speed below the stall limit, the gear dropped to 0.
//!
//! Caller: `physics::car_physics_step` (0x43B966), every physics step.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use rc2k_core::physics::gearing::drops_to_neutral;

/// A compare of floats only: the same at any precision.
// Converted: `rc2k_core::physics::gearing::drops_to_neutral`.
pub fn drop_to_neutral(car: &Car, tables: &PhysicsTables) {
    if !drops_to_neutral(
        car.contact_flag.get(),
        car.gear.get(),
        f32::from_bits(car.engine_speed.bits()),
        f32::from_bits(tables.stall_engine_speed.bits()),
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
