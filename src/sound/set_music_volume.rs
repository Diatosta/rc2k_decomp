//! `sound::set_music_volume` (0x40F83E): tells the music a new volume, if it is one.
//!
//! In:  eax = the volume
//! Out: every register preserved
//! Nothing if eax is what `dword_506994` holds; else command 0xB with eax = 0, ebx = 0,
//! ecx = the volume and edx = 3, then `sound::wait_for_commands`. That 3 means "the
//! volume" is a reading of its three callers (the options' volume steps and
//! `sub_43A649`).
//! Shadow mode runs the original alone.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::sound::senders::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::set_music_volume", rva(0x40F83E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The registers the command is sent with, or None if the volume is the one in use.
pub fn set_music_volume(entry: &Regs) -> Option<Regs> {
    (entry.eax != variable(0x506994)).then_some(Regs {
        eax: 0,
        ebx: 0,
        ecx: entry.eax,
        edx: 3,
        ..*entry
    })
}

pub const SENDER: Sender = Sender {
    address: 0x40F83E,
    len: 0x20,
    entry: [0x60, 0x3B, 0x05, 0x94, 0x69, 0x50, 0x00, 0x74],
    calls: &[0x14, 0x19],
    command: 0xB,
    sends: set_music_volume,
    reads: &[(0x506994, &[0, 0x40, 0x1234_5678])],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_sender(&HOOK, r, call_original, &SENDER);
}

// ---- self-test: a redirected copy whose command and wait note the registers ----

fn self_test() -> (usize, usize) {
    check_sender(&HOOK, &SENDER)
}
