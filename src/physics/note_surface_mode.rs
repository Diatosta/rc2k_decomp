//! `physics::note_surface_mode` (0x43F180): `dword_71A6C8` = 1 when `dword_71A65C` is 2
//! or 3, else 0. What the mode is is not known.
//!
//! Caller: `physics::feel_ground` (0x43F1A8), first.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::race::owner_shared::Shared;
use crate::session::Cell32;
use rc2k_core::physics::ground::surface_flag;

/// The Rust version on the variables' cells `v`.
// Converted: `rc2k_core::physics::ground::surface_flag`.
pub fn note_at(v: &[&dyn Cell32]) {
    v[1].set_bits(surface_flag(v[0].bits()));
}

fn game(shared: &Shared) -> Vec<&dyn Cell32> {
    vec![
        &shared.car.camera_view as &dyn Cell32,
        &shared.car.camera_mode_is_2_or_3,
    ]
}

/// The Rust version.
pub fn note_surface_mode(shared: &Shared) {
    note_at(&game(shared));
}
