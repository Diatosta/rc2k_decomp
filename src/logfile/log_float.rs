//! Logs the single-precision float whose bits are given,
//! with the given digits after the decimal point: a sign always ("+" or "-"), the integer part,
//! a dot and the decimals (nine at most; each bit's term is rounded down, so the last digits can
//! be a little low). Zero of either sign is "=0." and the decimals, without a sign; a number of
//! 2^32 or more is the sign and ">=2^32"; then the sign and "infinity" or "nan".

use crate::errors::BadInput;
use crate::logfile::numeric::{Body, NumberText, log_number};

pub fn float_text(bits: u32, decimals: u32) -> NumberText {
    let text = rc2k_core::files::log::float(bits, decimals);
    NumberText {
        bytes: text.bytes,
        nul_at: text.nul_at,
    }
}

fn text(bits: u32, decimals: u32) -> Result<NumberText, BadInput> {
    Ok(float_text(bits, decimals))
}

/// Logs `value` with `decimals` after the point.
pub fn log_float(value: f32, decimals: u32, newline: bool, level: u8) {
    log_number(&BODY, value.to_bits(), decimals, newline, level);
}

pub const BODY: Body = Body { text };
