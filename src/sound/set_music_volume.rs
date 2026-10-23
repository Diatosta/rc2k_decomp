//! `sound::set_music_volume` (0x40F83E): tells the music a new volume, if it is one.
//!
//! In:  eax = the volume
//! Out: every register preserved
//! Nothing if eax is what `dword_506994` holds; else command 0xB with eax = 0, ebx = 0,
//! ecx = the volume and edx = 3, then `sound::wait_for_commands`. That 3 means "the
//! volume" is a reading of its three callers (the options' volume steps and
//! `sub_43A649`).
//! Shadow mode runs the original alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::senders::*;

// ---- Rust version ----

/// The registers the command is sent with, or None if the volume is the one in use.
pub fn set_music_volume(entry: &Regs) -> Option<Regs> {
    // Converted: `rc2k_core::sound::senders::new_music_volume`.
    rc2k_core::sound::senders::new_music_volume(entry.eax, variable(0x506994)).then_some(Regs {
        eax: 0,
        ebx: 0,
        ecx: entry.eax,
        edx: 3,
        ..*entry
    })
}

pub const SENDER: Sender = Sender {
    command: 0xB,
    sends: set_music_volume,
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}
