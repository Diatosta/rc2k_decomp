//! The sound mode a voice's sample is prepared for. For a voice whose sound is not negative, by
//! its sample record: sound 0x1C4 keeps only the halvings (low four bits); groups 1 and 2 at least
//! one halving, group 3 at least two. Refuses a sound of 0x1CD or more.

use crate::audio::sample_records::SampleRecords;
use crate::audio::samples::sample_mode;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

pub(super) fn sample_mode_of(
    records: &SampleRecords,
    mode: u32,
    sound: u32,
) -> Result<u32, Refused> {
    if (sound as i32) < 0 {
        return Ok(mode);
    }
    if sound >= SAMPLES {
        return Err(BadInput("a sound of 0x1CD or more").into());
    }
    Ok(sample_mode(mode, Some(&sample(records, sound))))
}
