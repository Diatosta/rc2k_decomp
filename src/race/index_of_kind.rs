//! `race::index_of_kind` (0x437720): the place of car kind eax in `dword_6051E0` (0x22
//! kinds, the first of equals), 0 for none.
//!
//! In:  eax = the kind
//! Out: eax = the place (0 for none), ecx = the place (0x22 for none); the rest
//!      preserved
//!
//! See `race/race_shell.rs`. Callers: `sub_449B79` four times (each `mov`s eax away; no
//! flags read). Shadow mode: the Rust version and the original, registers compared.
//! Self-test: the game's table, every kind in it and some not, and a table of the test's
//! with repeats, put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, Regs};
use crate::race::race_shell::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { index_of_kind(r) };
    Ok(Left::nothing().with(EAX, g.eax).with(ECX, g.ecx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
