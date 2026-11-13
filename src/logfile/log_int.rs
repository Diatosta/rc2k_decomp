//! Logs the number as a signed decimal number.

use crate::errors::BadInput;
use crate::logfile::numeric::{Body, NumberText, log_number};

/// The text of a signed number, with `flags` for `format_u32`.
pub fn signed_text(value: u32, flags: u8) -> Vec<u8> {
    rc2k_core::files::log::signed(
        value as i32,
        rc2k_core::strings::format::Pad::from_flags(flags),
    )
}

fn text(value: u32, _: u32) -> Result<NumberText, BadInput> {
    Ok(NumberText::plain(signed_text(value, 0)))
}

/// Logs `value` as a signed decimal number.
pub fn log_int(value: i32, newline: bool, level: u8) {
    log_number(&BODY, value as u32, 0, newline, level);
}

pub const BODY: Body = Body { text };
