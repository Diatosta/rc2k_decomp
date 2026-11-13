//! Reads a sample record's file: its `data_key` is the file's path; none fails.

use crate::audio::voices::SampleRecord;
use crate::errors::Refused;

/// `read` reads a path; true for found. Returns true when it worked.
pub fn read_sample_of(
    record: &SampleRecord,
    read: &mut dyn FnMut(u32) -> Result<bool, Refused>,
) -> Result<bool, Refused> {
    if record.data_key == 0 {
        return Ok(false);
    }
    read(record.data_key)
}
