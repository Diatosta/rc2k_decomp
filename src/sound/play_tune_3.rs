//! `sound::play_tune_3` (0x40F780): starts tune 3.
//!
//! In:  nothing
//! Out: every register preserved
//! `sound::play_music` with tune 3, second tune 3 and the volume
//! `dword_5180FC` (the music's volume, by its use).
//! Called by `menu::play_films` from the idle menu, `sub_416F50`, four event starters.
//! Shadow mode runs the original alone.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::sound::senders::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_tune_3", rva(0x40F780), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The registers the music's command is sent with.
pub fn play_tune_3(entry: &Regs) -> Option<Regs> {
    tune_regs(entry, 3)
}

const SENDER: Sender = Sender {
    address: 0x40F780,
    len: 0x12,
    entry: [0x60, 0xB0, 0x03, 0xB4, 0x03, 0x8B, 0x0D, 0xFC],
    calls: &[0xB],
    command: 0xB,
    sends: play_tune_3,
    reads: &[(0x5180FC, &[0, 0x40, 0x7F])],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_sender(&HOOK, r, call_original, &SENDER);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}

// ---- self-test: a redirected copy whose command and wait note the registers ----

fn self_test() -> (usize, usize) {
    check_sender(&HOOK, &SENDER)
}
