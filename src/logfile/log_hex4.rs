//! `logfile::log_hex4` (0x401682): logs the low 4 hex digits of eax.
//!
//! In:  eax = the value, dl = 1 for a line break after it, dh = level - 1
//! Out: see `numeric.rs`
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{Regs, rva};
use crate::logfile::numeric::{Body, NumberText, log_number};
use crate::strings::hex_append::hex_append;

// ---- Rust version ----

fn text(r: &Regs) -> Result<NumberText, BadInput> {
    Ok(NumberText::plain(hex_append(r.eax, 4)?))
}

/// For Rust callers.
pub fn log_hex4(value: u32, newline: bool, level: u8) {
    log_number(&BODY, value, 0, newline, level);
}

pub const BODY: Body = Body {
    after_lock_call_rva: rva(0x401694),
    text,
};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::logfile::numeric::number_entry(r, &BODY)
}
