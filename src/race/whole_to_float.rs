//! `race::whole_to_float` (0x40D706): the dword at ebx made a float (`fild` / `fstp`) when
//! it looks like a whole number (top byte 0 or 0xFF).
//!
//! In:  ebx = the dword
//! Out: eax = the dword's top byte (`and 0xFF000000`); the rest preserved
//!
//! See `race/sound_tables.rs`. Callers: `race::sound_span`, `race::sound_row` (no flags
//! read). Shadow mode: the Rust version, the dword put back, the original's, compared.
//! Self-test: dwords round the test (0, small, negative, floats, the byte's edges).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs};
use crate::race::sound_tables::whole_to_float;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { whole_to_float(r.ebx) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
