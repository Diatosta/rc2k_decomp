//! `logfile::log_hex8` (0x4015F4): logs the low 8 hex digits of eax.
//!
//! In:  eax = the value, dl = 1 for a line break after it, dh = level - 1
//! Out: see `numeric.rs`
//! Hook removed 2026-10-22 (direct calls): Rust callers run the typed
//! function; calls by address `numeric::number_entry` (`routes.rs`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::rva;
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

pub const BODY: Body = Body {
    after_lock_call_rva: rva(0x401606),
    text,
};
