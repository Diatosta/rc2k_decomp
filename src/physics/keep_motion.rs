//! `physics::keep_motion` (0x440B7C): `dword_71A17C`, `dword_71A180`, `g_carSpeed`
//! (0x71A184) into `dword_61409C`, `dword_6140A0`, `dword_6140A4` (eax pushed around).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B87E), every physics step.

use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;

unsafe fn core(v: &[u32]) {
    for k in 0..3 {
        unsafe { (v[k + 3] as *mut u32).write_unaligned((v[k] as *const u32).read_unaligned()) };
    }
}

fn game() -> Vec<u32> {
    vec![
        CAR.local_velocity[0].addr() as u32,
        CAR.local_velocity[1].addr() as u32,
        CAR.local_velocity[2].addr() as u32,
        PHYSICS.kept_motion[0].addr() as u32,
        PHYSICS.kept_motion[1].addr() as u32,
        PHYSICS.kept_motion[2].addr() as u32,
    ]
}

/// The Rust version.
pub unsafe fn keep_motion() {
    unsafe { core(&game()) }
}
