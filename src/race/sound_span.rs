//! `race::sound_span` (0x40D71B): the two dwords at ebx made floats
//! (`race::whole_to_float`), the third their difference (`fld` / `fsub` / `fstp`).
//!
//! In:  ebx = the span
//! Out: eax = as the second `whole_to_float`, ebx = the second dword; the rest preserved
//!
//! See `race/sound_tables.rs`. Callers: `race::prepare_sound_tables`, `race::sound_row`.
//! It begins with its call: private copy, both calls relocated, manual patch. Shadow mode:
//! the Rust version (`sound_span_plain`), the span put back, the original's, compared
//! (the span within `FLOAT`). Self-test, at the test precisions: spans of
//! whole numbers and floats, eight ways.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, Regs};
use crate::race::sound_tables::sound_span_plain;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { sound_span_plain(r) };
    Ok(Left::nothing().with(EAX, g.eax).with(EBX, g.ebx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
