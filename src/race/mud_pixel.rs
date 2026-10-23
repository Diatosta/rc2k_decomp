//! `race::mud_pixel` (0x44A58A): a pixel of `g_surface` taken apart, or 0 outside it.
//!
//! In:  ecx = x, edx = y
//! Out: eax = the components; the rest preserved
//!
//! See `race/mud.rs`. One caller, `race::sample_mud_colours`. Self-test: surfaces of the
//! test's (16 and 32 bits), places inside and outside.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::race::mud::mud_pixel;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { mud_pixel(r.ecx, r.edx) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
