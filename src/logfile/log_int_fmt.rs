//! `logfile::log_int_fmt` (0x40148B): logs eax as a signed decimal number, formatted as the
//! `format_u32` flags in ecx say (0x10 zero-pad, 0x20 space-pad, low four bits the width).
//!
//! In:  eax = the number, ecx = flags, dl = 1 for a line break after it, dh = level - 1
//! Out: see `numeric.rs`
//! The minus sign comes before the padded magnitude.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{Regs, rva};
use crate::logfile::log_int::signed_text;
use crate::logfile::numeric::{Body, NumberText, log_number};

// ---- Rust version ----

fn text(r: &Regs) -> Result<NumberText, BadInput> {
    Ok(NumberText::plain(signed_text(r.eax, r.ecx as u8)))
}

/// For Rust callers.
pub fn log_int_fmt(value: i32, flags: u8, newline: bool, level: u8) {
    log_number(&BODY, value as u32, flags as u32, newline, level);
}

pub const BODY: Body = Body {
    after_lock_call_rva: rva(0x40149D),
    text,
};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::logfile::numeric::number_entry(r, &BODY)
}
