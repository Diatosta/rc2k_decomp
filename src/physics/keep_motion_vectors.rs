//! `physics::keep_motion_vectors` (0x43BD41): unless `dword_614148` is set, the three
//! dwords at `dword_71A170` (the vector `physics::store_motion_length` measures) into
//! `dword_614BCC` .. `D4`, and those at `dword_71A15C` into `dword_614BDC` .. `E4` (eax
//! pushed around).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B89B), every physics step.

use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;

unsafe fn core(v: &[u32]) {
    unsafe {
        if (v[0] as *const u32).read_unaligned() != 0 {
            return;
        }
        for k in 0..6 {
            (v[k + 7] as *mut u32).write_unaligned((v[k + 1] as *const u32).read_unaligned());
        }
    }
}

fn game() -> Vec<u32> {
    vec![
        PHYSICS.contact_step.addr() as u32,
        CAR.velocity[0].addr() as u32,
        CAR.velocity[1].addr() as u32,
        CAR.velocity[2].addr() as u32,
        CAR.spin[0].addr() as u32,
        CAR.spin[1].addr() as u32,
        CAR.spin[2].addr() as u32,
        PHYSICS.kept_vectors[0].addr() as u32,
        PHYSICS.kept_vectors[1].addr() as u32,
        PHYSICS.kept_vectors[2].addr() as u32,
        PHYSICS.kept_vectors[4].addr() as u32,
        PHYSICS.kept_vectors[5].addr() as u32,
        PHYSICS.kept_vectors[6].addr() as u32,
    ]
}

/// The Rust version.
pub unsafe fn keep_motion_vectors() {
    unsafe { core(&game()) }
}
