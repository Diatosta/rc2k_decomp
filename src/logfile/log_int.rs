//! `logfile::log_int` (0x401440): logs the number as a signed decimal number.
//!
//! Takes the number, 1 for a line break after it, level - 1. A minus sign and the magnitude through
//! `format_u32` with flags 0.

use crate::errors::BadInput;
use crate::logfile::numeric::{Body, NumberText, log_number};

// ---- Rust version ----

/// The text of a signed number, with `flags` for `format_u32`.
pub fn signed_text(value: u32, flags: u8) -> Vec<u8> {
    // Converted: `rc2k_core::files::log::signed`.
    rc2k_core::files::log::signed(
        value as i32,
        rc2k_core::strings::format::Pad::from_flags(flags),
    )
}

fn text(value: u32, _: u32) -> Result<NumberText, BadInput> {
    Ok(NumberText::plain(signed_text(value, 0)))
}

/// For Rust callers.
pub fn log_int(value: i32, newline: bool, level: u8) {
    log_number(&BODY, value as u32, 0, newline, level);
}

pub const BODY: Body = Body { text };
