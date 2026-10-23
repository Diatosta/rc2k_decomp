//! `sound::play_tune_6` (0x40F7C8): starts tune 6.
//!
//! In:  nothing
//! Out: every register preserved
//! `sound::play_music` with tune 6, second tune 0 and the volume
//! `dword_5180FC` (the music's volume, by its use).
//! Called by `sub_470F8A`, `sub_48AD73`.
//! Shadow mode runs the original alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::senders::*;

// ---- Rust version ----

/// The registers the music's command is sent with.
pub fn play_tune_6(entry: &Regs) -> Option<Regs> {
    tune_regs(entry, 6)
}

pub const SENDER: Sender = Sender {
    command: 0xB,
    sends: play_tune_6,
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}
