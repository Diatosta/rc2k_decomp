//! Logs the low 8 hex digits of the value.

use crate::errors::BadInput;
use crate::logfile::numeric::{Body, NumberText, log_number};
use crate::strings::hex_append::hex_append;

fn text(value: u32, _: u32) -> Result<NumberText, BadInput> {
    Ok(NumberText::plain(hex_append(value, 8)?))
}

/// Logs `value`'s eight hex digits.
pub fn log_hex8(value: u32, newline: bool, level: u8) {
    log_number(&BODY, value, 0, newline, level);
}

pub const BODY: Body = Body { text };
