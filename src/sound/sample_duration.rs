//! `sound::sample_duration` (0x40BA0B): how long a sample plays at a rate, in ms.
//!
//! Returns: the milliseconds
//! A sample without a buffer (+8 null): 0; a looping one (+0x14 is 1): 0x7FFFFFFF.
//! Else the buffer's +0x2C (its length) x 1000 (`imul`, the low 32 bits) over the rate
//! shifted right by the buffer's +0x28 & 0xF (unsigned `div`).
//! Three callers (the race sounds' turn). Refuses a sample past the records and a rate
//! that the shift makes 0 (the original's `div` traps).

use crate::errors::BadInput;
use crate::sound::sample_settings::{loaded, record};

pub fn sample_duration(
    audio: &crate::audio::owner::Audio,
    sample: u32,
    rate: u32,
) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::audio::samples::duration_ms`.
    let record = record(audio, sample)?;
    Ok(rc2k_core::audio::samples::duration_ms(
        rate,
        loaded(audio, &record).as_ref(),
    )?)
}
