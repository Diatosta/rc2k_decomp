//! `sound::play_tune_9` (0x40F7A4): starts tune 9.
//!
//! In:  nothing
//! Out: every register preserved
//! `sound::play_music` with tune 9, second tune 9 and the volume
//! `dword_5180FC` (the music's volume, by its use).
//! Called by `service::service_area_loop`.
//! Shadow mode runs the original alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::senders::*;

// ---- Rust version ----

/// The registers the music's command is sent with.
pub fn play_tune_9(entry: &Regs) -> Option<Regs> {
    tune_regs(entry, 9)
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}
