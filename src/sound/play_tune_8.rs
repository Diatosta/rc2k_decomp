//! `sound::play_tune_8` (0x40F7EC): starts tune 8, or stops the music.
//!
//! In:  nothing
//! Out: every register preserved
//! If `dword_5180F4` is 1: `sound::play_music` with tune 8, second tune 0xA and the
//! volume `dword_518100`; else `sound::stop_music`. One caller, `sub_470378`.
//! Shadow mode runs the original alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::senders::*;

// ---- Rust version ----

/// The registers the music's command is sent with: the tune's, or those that stop it.
pub fn play_tune_8(entry: &Regs) -> Option<Regs> {
    tune_regs(entry, 8)
}

pub const SENDER: Sender = Sender {
    address: 0x40F7EC,
    len: 0x22,
    entry: [0x60, 0x83, 0x3D, 0xF4, 0x80, 0x51, 0x00, 0x01],
    calls: &[0x14, 0x1B],
    command: 0xB,
    sends: play_tune_8,
    reads: &[(0x5180F4, &[0, 1, 2]), (0x518100, &[0, 0x33])],
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}
