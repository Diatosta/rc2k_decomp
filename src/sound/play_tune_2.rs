//! `sound::play_tune_2` (0x40F76F): starts tune 2.
//!
//! In:  nothing
//! Out: every register preserved
//! `sound::play_music` with tune 2, second tune 0 and the volume
//! 0x40.
//! Called by `sub_415FC2`.
//! Shadow mode runs the original alone.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::sound::senders::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_tune_2", rva(0x40F76F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The registers the music's command is sent with.
pub fn play_tune_2(entry: &Regs) -> Option<Regs> {
    tune_regs(entry, 2)
}

const SENDER: Sender = Sender {
    address: 0x40F76F,
    len: 0x11,
    entry: [0x60, 0xB0, 0x02, 0x32, 0xE4, 0xB9, 0x40, 0x00],
    calls: &[0xA],
    command: 0xB,
    sends: play_tune_2,
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
