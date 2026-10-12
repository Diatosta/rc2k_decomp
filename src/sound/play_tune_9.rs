//! `sound::play_tune_9` (0x40F7A4): starts tune 9.
//!
//! In:  nothing
//! Out: every register preserved
//! `sound::play_music` with tune 9, second tune 9 and the volume
//! `dword_5180FC` (the music's volume, by its use).
//! Called by `service::service_area_loop`.
//! Shadow mode runs the original alone.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::sound::senders::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_tune_9", rva(0x40F7A4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The registers the music's command is sent with.
pub fn play_tune_9(entry: &Regs) -> Option<Regs> {
    Some(music(entry, 9, 9, variable(0x5180FC)))
}

const SENDER: Sender = Sender {
    address: 0x40F7A4,
    len: 0x12,
    entry: [0x60, 0xB0, 0x09, 0xB4, 0x09, 0x8B, 0x0D, 0xFC],
    calls: &[0xB],
    command: 0xB,
    sends: play_tune_9,
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
