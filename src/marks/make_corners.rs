//! `marks::make_corners` (0x4BEB25): the two corner tables made from shape set 0 turned at
//! 0x81 eased steps.
//!
//! In:  nothing
//! Out: eax 0x102, esi the kind's box, the rest as the last corner leaves them
//!
//! See `marks/corners.rs`. One caller, `marks::make_masks`. Refuses a kind of 0x22 or
//! more, a farthest pair past 0x3F, what `marks::turn_shape_points` refuses at any step,
//! before writing. Self-test: the original and the Rust version on sets, pairs and steps
//! by lot, the refusal; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::corners::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { make_corners(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
