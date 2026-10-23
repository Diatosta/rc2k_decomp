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
    command: 0xB,
    sends: play_tune_4,
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}
