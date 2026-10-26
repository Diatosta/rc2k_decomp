//! `logfile::log_int` (0x401440): logs eax as a signed decimal number.
//!
//! In:  eax = the number, dl = 1 for a line break after it, dh = level - 1
//! Out: see `numeric.rs`
//! A minus sign and the magnitude through `format_u32` with flags 0.
//! Hook removed 2026-10-22 (direct calls): Rust callers run the typed
//! function; calls by address `numeric::number_entry` (`routes.rs`); the registers above are the original's, for reference.

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
