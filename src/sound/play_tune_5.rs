//! `sound::play_tune_5` (0x40F7B6): starts tune 5.
//!
//! In:  nothing
//! Out: every register preserved
//! `sound::play_music` with tune 5, second tune 0 and the volume
//! `dword_5180FC` (the music's volume, by its use).
//! Called by `sub_470F8A`.
//! Shadow mode runs the original alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::senders::*;

// ---- Rust version ----

/// The registers the music's command is sent with.
pub fn play_tune_5(entry: &Regs) -> Option<Regs> {
    tune_regs(entry, 5)
}

pub const SENDER: Sender = Sender {
    address: 0x40F7B6,
    len: 0x12,
    entry: [0x60, 0xB0, 0x05, 0x32, 0xE4, 0x8B, 0x0D, 0xFC],
    calls: &[0xB],
    command: 0xB,
    sends: play_tune_5,
    reads: &[(0x5180FC, &[0, 0x40, 0x7F])],
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}
