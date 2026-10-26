//! `sound::play_tune_2` (0x40F76F): starts tune 2.
//!
//! In:  nothing
//! Out: every register preserved
//! `sound::play_music` with tune 2, second tune 0 and the volume
//! 0x40.
//! Called by `sub_415FC2`.
//! Shadow mode runs the original alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::senders::*;

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}
