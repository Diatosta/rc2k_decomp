//! `marks::set_up_hexagons` (0x4BAA55): in `sub_4C0F2E`, the items cleared and the tables of
//! six-point circles, 0x40 triangle fans and their records (`g_marksHexagonItems` .., 0x800 of 0x30).
//!
//! In:  nothing
//! Out: see `marks/shapes.rs`
//!
//! See `marks/shapes.rs` (the once-only scaling of four floats at 0x816192 done the first time). One caller, `sub_4C0F2E`. Refuses a once-only mark
//! neither 0 nor where it goes on. Self-test: the original and the Rust version with the
//! mark 0 and set, the game's radii and radii by lot, at the test control words;
//! everything put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr};
use crate::marks::shapes::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let resume = game_addr(HEXAGONS.resume) as u32;
    Ok(left_of(&unsafe { set_up_hexagons_plain(r, resume) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
