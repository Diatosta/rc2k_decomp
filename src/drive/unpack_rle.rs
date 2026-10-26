//! `drive::unpack_rle` (0x41F080): run-length unpacking from esi to edi: a signed count
//! byte; below 0, that many bytes copied; above 0, the next byte repeated count + 1
//! times; 0 ends it.
//!
//! In:  esi the packed bytes, edi where they go
//! Out: ecx the bytes written; eax, esi, edi kept (pushed); no flags read
//!
//! Caller: `drive::read_packed`. A loop runs back to its fourth byte: hooked by a private
//! copy (0x23 bytes, no calls) and a hand-made patch.
//! Refuses (before anything) more than 0x100000 bytes out (the original runs on: a guard
//! against a broken frame).
//! Self-test: the original and the Rust version on test runs (copies, repeats, both,
//! empty) into a test buffer.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::bridge::Raw;
use crate::errors::Refused;
use crate::framework::{ECX, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::drive::rle;

/// The Rust version: the bytes written.
// Converted: `rc2k_core::drive::rle::unpack_rle`.
pub unsafe fn unpack_rle(from: u32, to: u32) -> Result<u32, Refused> {
    let packed = unsafe { Raw::new(from as *const u8) };
    Ok(rle::unpack_rle(&packed, |k, b| unsafe {
        ((to + k) as *mut u8).write(b)
    })?)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(ECX, unsafe { unpack_rle(r.esi, r.edi) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
