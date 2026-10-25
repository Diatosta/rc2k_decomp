//! `physics::note_surface_mode` (0x43F180): `dword_71A6C8` = 1 when `dword_71A65C` is 2
//! or 3, else 0. What the mode is is not known.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `physics::feel_ground` (0x43F1A8), first.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::car_store::CAR;
use crate::physics::cells::{get, put};
use rc2k_core::physics::ground::surface_flag;

/// The Rust version on the variables at `v`.
// Converted: `rc2k_core::physics::ground::surface_flag`.
pub unsafe fn note_at(v: &[u32]) {
    unsafe { put(v[1], surface_flag(get(v[0]))) }
}

fn game() -> Vec<u32> {
    vec![
        CAR.camera_view.addr() as u32,
        CAR.camera_mode_is_2_or_3.addr() as u32,
    ]
}

/// The Rust version.
pub unsafe fn note_surface_mode() {
    unsafe { note_at(&game()) }
}
