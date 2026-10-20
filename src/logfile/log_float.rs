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

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::logfile::numeric::{Body, NumberText, check_number, dispatch_number, log_number};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("logfile::log_float", rva(0x4014D8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    log_lock
);

// ---- Rust version ----

pub fn float_text(bits: u32, decimals: u32) -> NumberText {
    // Converted: `rc2k_core::files::log::float`.
    let text = rc2k_core::files::log::float(bits, decimals);
    NumberText {
        bytes: text.bytes,
        nul_at: text.nul_at,
    }
}

fn text(r: &Regs) -> Result<NumberText, BadInput> {
    Ok(float_text(r.eax, r.ecx))
}

/// For Rust callers.
pub unsafe fn log_float(value: f32, decimals: u32, newline: bool, level: u8) {
    unsafe { log_number(&BODY, value.to_bits(), decimals, newline, level) };
}

pub const BODY: Body = Body {
    after_lock_call_rva: rva(0x4014EA),
    text,
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_number(&HOOK, r, call_original, &BODY);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_number(
        &mut t,
        call_original,
        &BODY,
        &[
            (0.0f32.to_bits(), 3),
            (0x8000_0000, 2),
            (1.0f32.to_bits(), 3),
            ((-1.5f32).to_bits(), 1),
            (std::f32::consts::PI.to_bits(), 5),
            (0.1f32.to_bits(), 9),
            (123_456.79f32.to_bits(), 2),
            (0.001f32.to_bits(), 6),
            (0.999_999f32.to_bits(), 0),
            (4.0e9f32.to_bits(), 1),
            (4_294_967_296.0f32.to_bits(), 3),
            ((-1.0e20f32).to_bits(), 3),
            (f32::INFINITY.to_bits(), 3),
            (f32::NEG_INFINITY.to_bits(), 3),
            (f32::NAN.to_bits(), 3),
            (1, 9),
            (0x0080_0000, 9),
            (2.5f32.to_bits(), 12),
        ],
    );
    t.finish()
}
