//! `logfile::log_int_fmt` (0x40148B): logs eax as a signed decimal number, formatted as the
//! `format_u32` flags in ecx say (0x10 zero-pad, 0x20 space-pad, low four bits the width).
//!
//! In:  eax = the number, ecx = flags, dl = 1 for a line break after it, dh = level - 1
//! Out: see `numeric.rs`
//! The minus sign comes before the padded magnitude.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::logfile::log_int::signed_text;
use crate::logfile::numeric::{Body, NumberText, check_number, dispatch_number, log_number};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("logfile::log_int_fmt", rva(0x40148B), thunk, &ORIGINAL)
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

fn text(r: &Regs) -> Result<NumberText, BadInput> {
    Ok(NumberText::plain(signed_text(r.eax, r.ecx as u8)))
}

/// For Rust callers.
pub unsafe fn log_int_fmt(value: i32, flags: u8, newline: bool, level: u8) {
    unsafe { log_number(&BODY, value as u32, flags as u32, newline, level) };
}

pub const BODY: Body = Body {
    after_lock_call_rva: rva(0x40149D),
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
            (7, 0x12),
            (7, 0x24),
            (0xFFFF_FFF9, 0x13),
            (123_456, 0x18),
            (123_456, 0x23),
            (5, 0x15),
        ],
    );
    t.finish()
}
