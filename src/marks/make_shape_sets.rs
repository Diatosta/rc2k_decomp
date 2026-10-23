//! `marks::make_shape_sets` (0x4BD019): a shape set made for each of the settings' objects.
//!
//! In:  nothing
//! Out: esi the settings, the rest as the last set leaves them (eax the kind with none)
//!
//! See `marks/shape_points.rs`. One caller, `marks::make_masks`. Refuses a kind of 0x22 or
//! more, more than 19 objects, what `make_shape_set` refuses for any of them, before
//! writing. Self-test: the original and the Rust version on the scene of
//! `marks::measure_lines`, 0, 1 and 3 objects, the refusal; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::shape_points::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { make_shape_sets(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
