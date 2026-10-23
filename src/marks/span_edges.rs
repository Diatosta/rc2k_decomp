//! `marks::span_edges` (0x4BE315): a row of spans at edi for each whole row between the
//! edge pair's two heights.
//!
//! In:  edi where the spans go; the edge pair at `dword_82FD16` ..
//! Out: eax and ebx the last check's row and end, edi past the rows
//!
//! See `marks/spans.rs`. Callers: `marks::span_triangle` (twice). Refuses rows past the
//! work block or more than 0x1000, before writing. Self-test: the original and the Rust
//! version on edge pairs by lot (with the x87 as the game has it, and rounding toward
//! zero as the triangle sets it), the refusal; put back afterwards.
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
    Ok(left_of(&unsafe { span_edges(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
