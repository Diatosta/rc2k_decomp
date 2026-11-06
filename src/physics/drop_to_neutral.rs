//! `physics::drop_to_neutral` (0x43FD71): with `dword_71A20C` set, in gear, the engine
//! speed below `flt_6140FC` (`fcomp`, C0: also when unordered) and `dword_61416C` 0: gear
//! 0, `dword_71A1A8` = `dword_613FA4`, `dword_71A1A4` = 0x14, `dword_71A210` and
//! `dword_71A214` 1 (as the gear changes of `sub_43C20D` set them). By the look of it the
//! engine stalling into neutral (a reading).
//!
//! In:  nothing
//! Out: every register kept (eax pushed around `fnstsw`)
//!
//! Caller: `sub_43B635` (0x43B966), every physics step.

use crate::race::owner::race;
use rc2k_core::physics::gearing::drops_to_neutral;

/// The Rust version (a compare of floats only: the same at any precision).
// Converted: `rc2k_core::physics::gearing::drops_to_neutral`.
pub fn drop_to_neutral() {
    let shared_store = &race().shared;

    if !drops_to_neutral(
        shared_store.car.contact_flag.get(),
        shared_store.car.gear.get(),
        f32::from_bits(shared_store.car.engine_speed.bits()),
        f32::from_bits(shared_store.physics_tables.stall_engine_speed.bits()),
        || shared_store.physics_tables.no_auto_neutral.get(),
    ) {
        return;
    }
    shared_store.car.gear.set(0);
    shared_store
        .car
        .drive_cut_steps
        .set(shared_store.physics_tables.drive_cut_steps.get());
    shared_store.car.gear_change_hold.set(0x14);
    shared_store.car.gear_changed.set(1);
    shared_store.car.gear_changed_2.set(1);
}
