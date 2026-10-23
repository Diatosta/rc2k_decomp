//! `race::pack_runs` (0x41EFEC): ecx bytes at esi packed to edi by runs: 2 .. 0x80 equal
//! bytes as (length - 1, the byte), other stretches of up to 0x80 as (-length, the
//! bytes); fewer than three bytes or one left as one literal; then a dword 0 of which
//! one byte counts.
//!
//! In:  esi = the bytes, ecx = their count, edi = where the packed bytes go
//! Out: ecx = the bytes written; the rest preserved
//!
//! See `race/save_seal.rs`. One caller, `sub_41EEE4` (the results file; `add ecx, 3`
//! follows, no flags read). Shadow mode: the Rust version into a buffer of its own, the
//! original into the caller's, the two compared (the caller's buffer's size is not
//! known, so it is not captured). Self-test: sources of the test's (runs of 1 .. 300,
//! literal stretches past 0x80, mixes, 0 .. 3 bytes), buffers compared whole.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::save_seal::pack_runs;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    r.ecx = unsafe { pack_runs(r.esi, r.ecx, r.edi) };
    Ok(())
}
