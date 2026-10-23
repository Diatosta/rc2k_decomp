//! `race::invert_car_matrix` (0x43F16C): the work car's matrix (`g_crashMatrix`,
//! 0x71A08C, `g_car` +0xB4) undone by `scene::invert_turn` into `g_carInverseMatrix`
//! (0x71A9A4, +0x9CC); `pusha` .. `popa`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/start_place.rs`. Callers: `sub_43FB47` (race init's camera start, and three
//! more) and `sub_43B635` three times (the physics, stream B's). Shadow mode: the Rust
//! version, the matrices and `invert_turn`'s work put back, the original's, compared.
//! Self-test: the game's car matrix set to matrices by lot, both versions, put back
//! afterwards; every place is a float, held bit for bit but for a NaN's bits
//! (`Tolerance::Exact`: with shadow off the original's `invert_turn` is the converted one).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::start_place::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { invert_car_matrix() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
