//! `logfile::log_hex8` (0x4015F4): logs the low 8 hex digits of eax.
//!
//! In:  eax = the value, dl = 1 for a line break after it, dh = level - 1
//! Out: see `numeric.rs`

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::logfile::numeric::{Body, NumberText, check_number, dispatch_number, log_number};
use crate::strings::hex_append::hex_append;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("logfile::log_hex8", rva(0x4015F4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    log_lock
);

// ---- Rust version ----

fn text(r: &Regs) -> Result<NumberText, BadInput> {
    Ok(NumberText::plain(hex_append(r.eax, 8)?))
}

/// For Rust callers.
pub unsafe fn log_hex8(value: u32, newline: bool, level: u8) {
    unsafe { log_number(&BODY, value, 0, newline, level) };
}

pub const BODY: Body = Body {
    after_lock_call_rva: rva(0x401606),
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
            (0xDEAD_BEEF, 0),
            (0xFFFF_FFFF, 0),
            (0x1234_5678, 0),
            (0xAB, 0),
        ],
    );
    t.finish()
}
