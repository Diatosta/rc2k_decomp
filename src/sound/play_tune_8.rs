//! `sound::play_tune_8` (0x40F7EC): starts tune 8, or stops the music.
//!
//! In:  nothing
//! Out: every register preserved
//! If `dword_5180F4` is 1: `sound::play_music` with tune 8, second tune 0xA and the
//! volume `dword_518100`; else `sound::stop_music`. One caller, `sub_470378`.
//! Shadow mode runs the original alone.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::sound::senders::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_tune_8", rva(0x40F7EC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The registers the music's command is sent with: the tune's, or those that stop it.
pub fn play_tune_8(entry: &Regs) -> Option<Regs> {
    Some(if variable(0x5180F4) == 1 {
        music(entry, 8, 0xA, variable(0x518100))
    } else {
        Regs {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
            ..*entry
        }
    })
}

const SENDER: Sender = Sender {
    address: 0x40F7EC,
    len: 0x22,
    entry: [0x60, 0x83, 0x3D, 0xF4, 0x80, 0x51, 0x00, 0x01],
    calls: &[0x14, 0x1B],
    command: 0xB,
    sends: play_tune_8,
    reads: &[(0x5180F4, &[0, 1, 2]), (0x518100, &[0, 0x33])],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_sender(&HOOK, r, call_original, &SENDER);
}

// ---- self-test: a redirected copy whose command and wait note the registers ----

fn self_test() -> (usize, usize) {
    check_sender(&HOOK, &SENDER)
}
