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

use crate::errors::BadInput;
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
