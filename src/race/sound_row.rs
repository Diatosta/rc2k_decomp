//! `race::sound_row` (0x40D731): a row of a sound record at edx: +4, +8, +0xC made
//! floats (`race::whole_to_float`), spans at +0x10 and +0x1C (`race::sound_span`).
//!
//! In:  edx = the row
//! Out: eax = as the last `whole_to_float`, ebx = edx + 0x20; the rest preserved
//!
//! See `race/sound_tables.rs`. One caller, `race::prepare_sound_tables`. A call in its
//! first bytes: private copy, its five calls relocated, manual patch. Shadow mode: the
//! Rust version (`sound_row_plain`), the row put back, the original's, compared (the
//! spans within `FLOAT`). Self-test, at the test precisions: rows of whole
//! numbers and floats, eight ways.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, Regs};
use crate::race::sound_tables::sound_row_plain;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { sound_row_plain(r) };
    Ok(Left::nothing().with(EAX, g.eax).with(EBX, g.ebx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
