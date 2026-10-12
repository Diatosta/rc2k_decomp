//! `sound::play_music` (0x40F80E): starts a tune.
//!
//! In:  al = the tune, ah = a second tune, ecx = the volume
//! Out: every register preserved
//! Command 0xB with eax = al, ebx = ah, ecx as given and edx = 2, then
//! `sound::wait_for_commands`. Eight callers, the functions that each start one tune.
//! What the second tune is for (the one to go on with, by the look of the pairs) is not
//! read.
//! Shadow mode runs the original alone.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::sound::senders::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_music", rva(0x40F80E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The registers the command is sent with.
pub fn play_music(entry: &Regs) -> Option<Regs> {
    Some(music(
        entry,
        entry.eax & 0xFF,
        entry.eax >> 8 & 0xFF,
        entry.ecx,
    ))
}

const SENDER: Sender = Sender {
    address: 0x40F80E,
    len: 0x18,
    entry: [0x60, 0x0F, 0xB6, 0xDC, 0x0F, 0xB6, 0xC0, 0xBA],
    calls: &[0xC, 0x11],
    command: 0xB,
    sends: play_music,
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
