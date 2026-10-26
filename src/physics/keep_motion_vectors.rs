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
        if (crate::pointers::host(v[0]) as *const u32).read_unaligned() != 0 {
            return;
        }
        for k in 0..6 {
            (crate::pointers::host(v[k + 7]) as *mut u32)
                .write_unaligned((crate::pointers::host(v[k + 1]) as *const u32).read_unaligned());
        }
    }
}

fn game() -> Vec<u32> {
    vec![
        PHYSICS.contact_step.addr(),
        CAR.velocity[0].addr(),
        CAR.velocity[1].addr(),
        CAR.velocity[2].addr(),
        CAR.spin[0].addr(),
        CAR.spin[1].addr(),
        CAR.spin[2].addr(),
        PHYSICS.kept_vectors[0].addr(),
        PHYSICS.kept_vectors[1].addr(),
        PHYSICS.kept_vectors[2].addr(),
        PHYSICS.kept_vectors[4].addr(),
        PHYSICS.kept_vectors[5].addr(),
        PHYSICS.kept_vectors[6].addr(),
    ]
}

/// The Rust version.
pub unsafe fn keep_motion_vectors() {
    unsafe { core(&game()) }
}
