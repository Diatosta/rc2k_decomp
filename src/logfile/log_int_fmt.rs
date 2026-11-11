//! `logfile::log_int_fmt` (0x40148B): logs the number as a signed decimal number, formatted as the
//! `format_u32` flags say (0x10 zero-pad, 0x20 space-pad, low four bits the width).
//!
//! Takes the number, the flags, 1 for a line break after it and the level - 1. See `numeric.rs`.
//! The minus sign comes before the padded magnitude.

use crate::errors::BadInput;
use crate::logfile::log_int::signed_text;
use crate::logfile::numeric::{Body, NumberText, log_number};

// ---- Rust version ----

fn text(value: u32, flags: u32) -> Result<NumberText, BadInput> {
    Ok(NumberText::plain(signed_text(value, flags as u8)))
}

/// For Rust callers.
pub fn log_int_fmt(value: i32, flags: u8, newline: bool, level: u8) {
    log_number(&BODY, value as u32, flags as u32, newline, level);
}

pub const BODY: Body = Body { text };
