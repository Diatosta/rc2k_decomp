//! `logfile::log_int` (0x401440): logs eax as a signed decimal number.
//!
//! In:  eax = the number, dl = 1 for a line break after it, dh = level - 1
//! Out: see `numeric.rs`
//! A minus sign and the magnitude through `format_u32` with flags 0.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::logfile::numeric::{Body, NumberText, check_number, dispatch_number, log_number};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("logfile::log_int", rva(0x401440), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    log_lock
);

// ---- Rust version ----

/// The text of a signed number, with `flags` for `format_u32`.
pub fn signed_text(value: u32, flags: u8) -> Vec<u8> {
    // Converted: `rc2k_core::log::signed`.
    rc2k_core::log::signed(value as i32, rc2k_core::format::Pad::from_flags(flags))
}

fn text(r: &Regs) -> Result<NumberText, BadInput> {
    Ok(NumberText::plain(signed_text(r.eax, 0)))
}

/// For Rust callers.
pub unsafe fn log_int(value: i32, newline: bool, level: u8) {
    unsafe { log_number(&BODY, value as u32, 0, newline, level) };
}

pub const BODY: Body = Body {
    after_lock_call_rva: rva(0x40144E),
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
            (0, 0),
            (1, 0),
            (42, 0),
            (0xFFFF_FFFF, 0),
            (0x8000_0000, 0),
            (0x7FFF_FFFF, 0),
            (123_456_789, 0),
        ],
    );
    t.finish()
}
