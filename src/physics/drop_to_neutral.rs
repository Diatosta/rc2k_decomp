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

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core on the variables at `v` (in `VARIABLES`' order; a compare of floats
/// only: the same at any precision).
// Converted: `rc2k_core::physics::gearing::drops_to_neutral`.
pub unsafe fn drop_to_neutral_at(v: &[u32; 10]) {
    unsafe {
        let f = |k: usize| f32::from_bits(at(v[k]));
        if !drops_to_neutral(at(v[0]), at(v[1]), f(2), f(3), || at(v[4])) {
            return;
        }
        put(v[1], 0);
        put(v[6], at(v[5]));
        put(v[7], 0x14);
        put(v[8], 1);
        put(v[9], 1);
    }
}

fn game() -> [u32; 10] {
    [
        CAR.contact_flag.addr() as u32,
        CAR.gear.addr() as u32,
        CAR.engine_speed.addr() as u32,
        PHYSICS.stall_engine_speed.addr() as u32,
        PHYSICS.no_auto_neutral.addr() as u32,
        PHYSICS.drive_cut_steps.addr() as u32,
        CAR.drive_cut_steps.addr() as u32,
        CAR.gear_change_hold.addr() as u32,
        CAR.gear_changed.addr() as u32,
        CAR.gear_changed_2.addr() as u32,
    ]
}

/// The Rust version.
pub unsafe fn drop_to_neutral() {
    unsafe { drop_to_neutral_at(&game()) }
}
