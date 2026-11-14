//! `sound::sample_duration`: how long a sample plays at a rate, in ms.
//!
//! A sample without a buffer: 0; a looping one: 0x7FFFFFFF. Else the buffer's length x 1000 (the
//! low 32 bits) over the rate shifted right by the buffer's shift (its low four bits; unsigned
//! division). Three callers (the race sounds' turn). Refuses a sample past the records and a rate
//! that the shift makes 0.

use crate::errors::BadInput;
use crate::sound::sample_settings::{loaded, record};

pub fn sample_duration(
    audio: &crate::audio::owner::Audio,
    sample: u32,
    rate: u32,
) -> Result<u32, BadInput> {
    // The logic is `rc2k_core::audio::samples::duration_ms`.
    let record = record(audio, sample)?;
    Ok(rc2k_core::audio::samples::duration_ms(
        rate,
        loaded(audio, &record).as_ref(),
    )?)
}
