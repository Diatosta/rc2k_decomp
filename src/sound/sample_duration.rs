//! `sound::sample_duration` (0x40BA0B): how long a sample plays at a rate, in ms.
//!
//! In:  eax the sample (`g_sampleRecords`), edx the rate (-0x6F the record's +0x30, then
//!      -0x70 the buffer's +0x24, held to 100 .. 200000: `sound/sample_settings.rs`)
//! Out: eax the milliseconds; everything else preserved
//! A sample without a buffer (+8 null): 0; a looping one (+0x14 is 1): 0x7FFFFFFF.
//! Else the buffer's +0x2C (its length) x 1000 (`imul`, the low 32 bits) over the rate
//! shifted right by the buffer's +0x28 & 0xF (unsigned `div`).
//! Three callers (the race sounds' turn). Refuses a sample past the records and a rate
//! that the shift makes 0 (the original's `div` traps).
//! Shadow mode: both, the registers compared (`dispatch_state`, nothing written).
//! Self-test: every record with a set of rates, and record 0 as the test's (put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use crate::sound::sample_settings::{loaded, record};

// ---- Rust version ----

pub fn sample_duration(sample: u32, rate: u32) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::audio::samples::duration_ms`.
    let record = record(sample)?;
    Ok(rc2k_core::audio::samples::duration_ms(
        rate,
        loaded(record).as_ref(),
    )?)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, sample_duration(r.eax, r.edx)?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
