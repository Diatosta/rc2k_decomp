//! `physics::clear_records` (0x43F3A6): ecx records of 0x20 bytes from edi, the first six
//! dwords of each 0 (`dec ecx; jnz` back to its own first instruction).
//!
//! In:  edi the first record, ecx their number
//! Out: edi past the last, ecx 0; the rest kept; ZF set (`dec`), not read (its callers
//!      return next, theirs go on with calls)
//!
//! Its loop goes back to its first byte: `.private_copy` (the whole function).
//! Callers: `physics::clear_forces` (0x614770, 0x17), `physics::clear_contacts`
//! (0x614A50, 7).
//! Refuses (before anything) ecx 0 (the original would loop 2^32 times) or above 0x100.
//! Self-test: a copy of the original on a test buffer, counts 1 .. 0x20.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{ECX, EDI, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version.
// Converted: `rc2k_core::physics::pushes::{record_count, CLEARED_DWORDS}`.
pub unsafe fn clear_records(at: u32, count: u32) -> Result<Left, Refused> {
    use rc2k_core::physics::pushes::{CLEARED_DWORDS, record_count};
    let count = record_count(count, "a record count of 0 or above 0x100")?;
    for k in 0..count {
        for d in 0..CLEARED_DWORDS {
            unsafe { ((at + 0x20 * k + 4 * d) as *mut u32).write_unaligned(0) };
        }
    }
    Ok(Left::nothing().with(EDI, at + 0x20 * count).with(ECX, 0))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { clear_records(r.edi, r.ecx) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
