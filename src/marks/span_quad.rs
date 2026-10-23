//! `marks::span_quad` (0x4BE0C6): the quad at ebp (four x, y floats) spanned as two triangles.
//!
//! In:  ebp the corners
//! Out: every register kept
//!
//! See `marks/spans.rs`. Callers: `sub_4BE592`. Refuses runs past their table, and spans past the
//! work block or away, before writing. Self-test: the original and the Rust version on
//! corners by lot (two at one height now and then) in a work block of the test's, the
//! refusal; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::spans::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { span_quad(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
