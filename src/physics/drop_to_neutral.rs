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

use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use rc2k_core::physics::gearing::drops_to_neutral;

/// The Rust version (a compare of floats only: the same at any precision).
// Converted: `rc2k_core::physics::gearing::drops_to_neutral`.
pub fn drop_to_neutral() {
    if !drops_to_neutral(
        CAR.contact_flag.get(),
        CAR.gear.get(),
        f32::from_bits(CAR.engine_speed.bits()),
        f32::from_bits(PHYSICS.stall_engine_speed.bits()),
        || PHYSICS.no_auto_neutral.get(),
    ) {
        return;
    }
    CAR.gear.set(0);
    CAR.drive_cut_steps.set(PHYSICS.drive_cut_steps.get());
    CAR.gear_change_hold.set(0x14);
    CAR.gear_changed.set(1);
    CAR.gear_changed_2.set(1);
}
