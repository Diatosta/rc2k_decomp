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
use crate::strings::format_u32::format_u32;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("logfile::log_float", rva(0x4014D8), thunk, &ORIGINAL)
    .self_test(self_test)
    .self_test_unpaused();

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- Rust version ----

/// The sign, or "=" and nothing else decided yet, with the 32.32 fixed-point magnitude; or a
/// text that is complete as it is.
enum Parts {
    Number { lead: u8, whole: u32, fraction: u32 },
    Special(Vec<u8>),
}

fn parts(bits: u32) -> Parts {
    if bits & 0x7FFF_FFFF == 0 {
        return Parts::Number {
            lead: b'=',
            whole: 0,
            fraction: 0,
        };
    }
    let sign = if (bits as i32) < 0 { b'-' } else { b'+' };
    let exponent = (bits >> 23) & 0xFF;
    let mut whole = (exponent != 0) as u32; // the hidden bit; none for a denormal
    let mut fraction = (bits & 0x7F_FFFF) << 9;
    let special = |text: &[u8]| {
        let mut out = vec![sign];
        out.extend_from_slice(text);
        Parts::Special(out)
    };
    if exponent == 0xFF {
        return special(if fraction == 0 { b"infinity" } else { b"nan" });
    }
    let shift = exponent as i32 - 0x7F;
    for _ in 0..shift.max(0) {
        let carry = whole >> 31;
        whole = whole << 1 | fraction >> 31;
        fraction <<= 1;
        if carry != 0 {
            return special(b">=2^32");
        }
    }
    for _ in 0..(-shift).max(0) {
        fraction = fraction >> 1 | whole << 31;
        whole >>= 1;
    }
    Parts::Number {
        lead: sign,
        whole,
        fraction,
    }
}

/// Nine decimal digits for a 32-bit binary fraction, as the original adds them up.
fn fraction_digits(mut fraction: u32) -> u32 {
    let (mut step, mut sum) = (500_000_000u32, 0u32);
    while fraction != 0 {
        if fraction & 0x8000_0000 != 0 {
            sum += step;
        }
        step >>= 1;
        fraction <<= 1;
    }
    sum
}

pub fn float_text(bits: u32, decimals: u32) -> NumberText {
    match parts(bits) {
        Parts::Special(text) => NumberText::plain(text),
        Parts::Number {
            lead,
            whole,
            fraction,
        } => {
            let mut bytes = vec![lead];
            bytes.extend(format_u32(whole, 0));
            bytes.push(b'.');
            let nul_at = bytes.len().wrapping_add(decimals as usize);
            bytes.extend(format_u32(fraction_digits(fraction), 0x19));
            NumberText { bytes, nul_at }
        }
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
