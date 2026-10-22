//! `sound::play_music` (0x40F80E): starts a tune.
//!
//! In:  al = the tune, ah = a second tune, ecx = the volume
//! Out: every register preserved
//! Command 0xB with eax = al, ebx = ah, ecx as given and edx = 2, then
//! `sound::wait_for_commands`. Eight callers, the functions that each start one tune.
//! What the second tune is for (the one to go on with, by the look of the pairs) is not
//! read.
//! Shadow mode runs the original alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::senders::*;

// ---- Rust version ----

/// The registers the command is sent with.
pub fn play_music(entry: &Regs) -> Option<Regs> {
    Some(music(
        entry,
        entry.eax & 0xFF,
        entry.eax >> 8 & 0xFF,
        entry.ecx,
    ))
}

const SENDER: Sender = Sender {
    address: 0x40F80E,
    len: 0x18,
    entry: [0x60, 0x0F, 0xB6, 0xDC, 0x0F, 0xB6, 0xC0, 0xBA],
    calls: &[0xC, 0x11],
    command: 0xB,
    sends: play_music,
    reads: &[],
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}
