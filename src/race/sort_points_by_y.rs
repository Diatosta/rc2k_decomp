//! `race::sort_points_by_y` (0x426BE6): a car's points sorted by y, upwards.
//!
//! In:  esi = the first point, edi = the last (12 bytes each)
//! Out: the registers as the sort leaves them (eax, ebx, ecx, edx, ebp, esi; edi kept)
//!
//! See `race/objects.rs` (`sort_points`: the original's steps kept). Called by
//! `race::set_up_car_points` and by itself. Manual patch with a private copy (a `jnb` in
//! its first bytes; its own two calls relocated to the hook). Self-test: point arrays of
//! the test's (2 .. 0x3D points, repeated keys, keys above 0x80000000).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::objects::{BY_Y, sort_points};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    // Converted: `rc2k_core::race::quick_sort::quick_sort`.
    unsafe { sort_points(BY_Y, &mut g)? };
    Ok(left_of(&g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
