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
use crate::session::Cell32;
use rc2k_core::physics::ground::surface_flag;

/// The Rust version on the variables' cells `v`.
// Converted: `rc2k_core::physics::ground::surface_flag`.
pub fn note_at(v: &[&dyn Cell32]) {
    v[1].set_bits(surface_flag(v[0].bits()));
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![&CAR.camera_view as &dyn Cell32, &CAR.camera_mode_is_2_or_3]
}

/// The Rust version.
pub fn note_surface_mode() {
    note_at(&game());
}
