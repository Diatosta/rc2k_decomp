//! `logfile::log_hex8` (0x4015F4): logs the low 8 hex digits of eax.
//!
//! Takes the value, 1 for a line break after it, level - 1.

use crate::errors::BadInput;
use crate::logfile::numeric::{Body, NumberText, log_number};
use crate::strings::hex_append::hex_append;

// ---- Rust version ----

fn text(value: u32, _: u32) -> Result<NumberText, BadInput> {
    Ok(NumberText::plain(hex_append(value, 8)?))
}

/// For Rust callers.
pub fn log_hex8(value: u32, newline: bool, level: u8) {
    log_number(&BODY, value, 0, newline, level);
}

pub const BODY: Body = Body { text };
