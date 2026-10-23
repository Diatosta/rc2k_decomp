//! `race::seed_car_randoms` (0x444BF6): each car block's +0x10 and +0xC one
//! `race::nonzero_random`.
//!
//! In:  nothing
//! Out: eax = the last number, ecx = 8, esi = the last car block; the rest preserved
//!
//! See `race/race_init.rs`. One caller, `sub_4462B9` (race init). Shadow mode: the Rust
//! version, the blocks and the generator's state put back, the original's, compared.
//! Self-test: the game's eight blocks filled with values of the test's, generator states
//! of the test's (0 among them), both versions, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, ESI, Regs};
use crate::race::race_init::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { seed_car_randoms(r) };
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(ECX, g.ecx)
        .with(ESI, g.esi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
