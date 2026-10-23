//! `marks::measure_starting_sums` (0x4D0DB5): in step 14, every block measured and its sums
//! kept as the ones the shares are made of.
//!
//! In:  nothing
//! Out: eax the first block's last sum moved, ebx 0, ebp that block (the settings with
//!      none); the rest as `marks::measure_lines` leaves them
//!
//! See `marks/lines.rs`. One caller, `marks::set_up_blocks`. Refuses as `measure_lines`
//! with every block. Self-test: the original and the Rust version on scenes of the test's
//! (0, 1, 3 and 8 blocks), the refusal; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::lines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { measure_starting_sums(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
