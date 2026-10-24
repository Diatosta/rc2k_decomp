//! `audio::read_sample_of` (0x40B1BE): a sample record's file read.
//!
//! In:  ebx = the sample record
//! Out: carry = not read (no path, or `audio::read_sample`'s carry); every register preserved
//!
//! The record's `data_key` is its file's path: none sets the carry, else
//! `audio::read_sample` on it. One caller, command 3 (`sub_40AEB9`).

use crate::audio::voices::SampleRecord;
use crate::errors::Refused;

// ---- Rust version ----

/// `read`: the reading of a path, true for found. Returns true for the carry clear.
pub unsafe fn read_sample_of(
    record: &SampleRecord,
    read: &mut dyn FnMut(u32) -> Result<bool, Refused>,
) -> Result<bool, Refused> {
    if record.data_key == 0 {
        return Ok(false);
    }
    read(record.data_key)
}
