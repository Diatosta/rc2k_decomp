//! `sound::stop_stream` (0x40F757): command 9 with four zeros, and the wait for it.
//!
//! In:  nothing
//! Out: every register preserved
//! Command 9 (the audio thread's `sub_40C194` with eax, ecx, edx) with eax, ebx, ecx and
//! edx = 0, then `sound::wait_for_commands`. What command 9 plays is not read; "stream"
//! and "stop" are guesses from command 0xB's shape. Nine callers (the event loops, four
//! screen loops).
//! Shadow mode runs the original alone.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::sound::senders::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::stop_stream", rva(0x40F757), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The registers the command is sent with.
pub fn stop_stream(entry: &Regs) -> Option<Regs> {
    Some(Regs {
        eax: 0,
        ebx: 0,
        ecx: 0,
        edx: 0,
        ..*entry
    })
}

const SENDER: Sender = Sender {
    address: 0x40F757,
    len: 0x18,
    entry: [0x60, 0x33, 0xC0, 0x33, 0xDB, 0x33, 0xC9, 0xBA],
    calls: &[0xC, 0x11],
    command: 0x9,
    sends: stop_stream,
    reads: &[],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_sender(&HOOK, r, call_original, &SENDER);
}

// ---- self-test: a redirected copy whose command and wait note the registers ----

fn self_test() -> (usize, usize) {
    check_sender(&HOOK, &SENDER)
}
