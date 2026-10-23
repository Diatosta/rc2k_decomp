//! `scene::identity_matrix3` (0x46574E): the 3 x 3 matrix at esi made the identity.
//!
//! In:  esi the matrix
//! Out: every register kept
//!
//! See `scene/matrix3.rs`. Callers: `sub_4BEFD2` and code outside a function. Self-test:
//! the original and the Rust version on a buffer of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { identity_matrix3(r.esi) };
    Ok(left_of(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
