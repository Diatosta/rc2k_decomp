//! `logfile::log_float` (0x4014D8): logs the single-precision float whose bits are in eax,
//! with ecx digits after the decimal point.
//!
//! In:  eax = the float's bits, ecx = decimals, dl = 1 for a line break after it,
//!      dh = level - 1
//! Out: see `numeric.rs`
//! The text: a sign, always ("+" or "-"), the integer part, a dot and decimals. The number is
//! taken apart by hand: the 24-bit mantissa is shifted into a 32.32 fixed-point value, the
//! integer half is printed with `format_u32`, and the fraction half is turned into nine
//! digits by adding 500000000 >> n for every set bit n (each term rounded down, so the last
//! digits can be a little low). The text is then cut after `decimals` digits by putting the
//! NUL there; with more than nine, whatever the scratch buffer held shows through.
//! Special texts: zero of either sign is "=0." and decimals, without a sign; a number of 2^32
//! or more is the sign and ">=2^32"; then the sign and "infinity" or "nan".
//! Hook removed 2026-10-22 (direct calls): Rust callers run the typed
//! function; calls by address `numeric::number_entry` (`routes.rs`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::logfile::numeric::{Body, NumberText, log_number};

// ---- Rust version ----

pub fn float_text(bits: u32, decimals: u32) -> NumberText {
    // Converted: `rc2k_core::files::log::float`.
    let text = rc2k_core::files::log::float(bits, decimals);
    NumberText {
        bytes: text.bytes,
        nul_at: text.nul_at,
    }
}

fn text(bits: u32, decimals: u32) -> Result<NumberText, BadInput> {
    Ok(float_text(bits, decimals))
}

/// For Rust callers.
pub fn log_float(value: f32, decimals: u32, newline: bool, level: u8) {
    log_number(&BODY, value.to_bits(), decimals, newline, level);
}

pub const BODY: Body = Body { text };
