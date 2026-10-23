//! `gfx::read_pixel` (0x406584): a pixel of `g_surface` taken apart.
//!
//! In:  ecx = x, edx = y
//! Out: eax = the components; the rest preserved
//!
//! See `race/mud.rs` (a 16-bit pixel keeps the row address's upper half in eax). Two
//! callers (`race::mud_pixel` and one more). Self-test: surfaces of the test's (16 and 32
//! bits), places inside them.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::race::mud::read_pixel;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { read_pixel(r.ecx, r.edx) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
