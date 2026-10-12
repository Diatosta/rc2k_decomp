//! `sound::play_tune_6` (0x40F7C8): starts tune 6.
//!
//! In:  nothing
//! Out: every register preserved
//! `sound::play_music` with tune 6, second tune 0 and the volume
//! `dword_5180FC` (the music's volume, by its use).
//! Called by `sub_470F8A`, `sub_48AD73`.
//! Shadow mode runs the original alone.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::sound::senders::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_tune_6", rva(0x40F7C8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The registers the music's command is sent with.
pub fn play_tune_6(entry: &Regs) -> Option<Regs> {
    Some(music(entry, 6, 0, variable(0x5180FC)))
}

const SENDER: Sender = Sender {
    address: 0x40F7C8,
    len: 0x12,
    entry: [0x60, 0xB0, 0x06, 0x32, 0xE4, 0x8B, 0x0D, 0xFC],
    calls: &[0xB],
    command: 0xB,
    sends: play_tune_6,
    reads: &[(0x5180FC, &[0, 0x40, 0x7F])],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_sender(&HOOK, r, call_original, &SENDER);
}

// ---- self-test: a redirected copy whose command and wait note the registers ----

fn self_test() -> (usize, usize) {
    check_sender(&HOOK, &SENDER)
}
