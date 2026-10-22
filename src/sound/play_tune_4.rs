//! `sound::play_tune_4` (0x40F792): starts tune 4.
//!
//! In:  nothing
//! Out: every register preserved
//! `sound::play_music` with tune 4, second tune 4 and the volume
//! `dword_5180FC` (the music's volume, by its use).
//! Called by `menu::main_menu_loop`, `sub_416F50`, `sub_4703C2`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::sound::senders::*;

// ---- Rust version ----

/// The registers the music's command is sent with.
pub fn play_tune_4(entry: &Regs) -> Option<Regs> {
    tune_regs(entry, 4)
}

const SENDER: Sender = Sender {
    address: 0x40F792,
    len: 0x12,
    entry: [0x60, 0xB0, 0x04, 0xB4, 0x04, 0x8B, 0x0D, 0xFC],
    calls: &[0xB],
    command: 0xB,
    sends: play_tune_4,
    reads: &[(0x5180FC, &[0, 0x40, 0x7F])],
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}
