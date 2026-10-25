//! `drive::clear_start_state` (0x4376E3): six of the work copy's start values 0:
//! `dword_71A234` (the grades' mode, `physics::grade_or_reset`), `dword_71A070`,
//! `dword_71AA88`, `flt_71A650`, `flt_71A658`, `dword_71A01C`.
//!
//! In:  nothing
//! Out: every register kept; no flags read (`drive::store_car` next)
//!
//! Caller: `sub_43703E` (0x4371BC, 0x43729B).

use crate::physics::car_store::CAR;

unsafe fn core(v: &[u32]) {
    for &a in v {
        unsafe { (a as *mut u32).write_unaligned(0) };
    }
}

fn game() -> Vec<u32> {
    vec![
        CAR.driver_kind.addr() as u32,
        CAR.checkpoint.addr() as u32,
        CAR.engine_sound_speed_int.addr() as u32,
        CAR.start_share.addr() as u32,
        CAR.start_blend.addr() as u32,
        CAR.unknown_44.addr() as u32,
    ]
}

/// The Rust version.
pub unsafe fn clear_start_state() {
    unsafe { core(&game()) }
}
