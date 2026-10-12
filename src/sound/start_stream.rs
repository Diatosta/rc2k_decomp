//! `sound::start_stream` (0x40F743): command 9 with edx = 2, and the wait for it.
//!
//! In:  eax, ecx (passed on in the command)
//! Out: every register preserved
//! Command 9 with eax and ecx as given, ebx = 0 and edx = 2, then
//! `sound::wait_for_commands`. The name is a guess, as `sound::stop_stream`'s. Seven
//! callers.
//! Shadow mode runs the original alone.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::sound::senders::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::start_stream", rva(0x40F743), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The registers the command is sent with.
pub fn start_stream(entry: &Regs) -> Option<Regs> {
    Some(Regs {
        ebx: 0,
        edx: 2,
        ..*entry
    })
}

const SENDER: Sender = Sender {
    address: 0x40F743,
    len: 0x14,
    entry: [0x60, 0x33, 0xDB, 0xBA, 0x02, 0x00, 0x00, 0x00],
    calls: &[0x8, 0xD],
    command: 0x9,
    sends: start_stream,
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
