//! `scene::multiply_matrices3` (0x4657EC): ebp = esi x edi, 3 x 3 matrices of floats.
//!
//! In:  esi, edi the two, ebp the result
//! Out: ecx and edx 0, edi and ebp 12 on, esi kept
//!
//! See `scene/matrix3.rs`. Callers: `sub_4B9FE6` (`marks::turn_spline_points`),
//! `sub_4BA0AE`, `sub_4BA16C`, `sub_4BD5ED` and code outside a function. Self-test: the
//! original and the Rust version on matrices by lot (the result apart, and on the first).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::*;
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { multiply_matrices3_plain(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
