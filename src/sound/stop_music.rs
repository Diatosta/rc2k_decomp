//! `sound::stop_music` (0x40F826): stops the music: command 0xB with four zeros, and the
//! wait for it.
//!
//! In:  nothing (esi, edi and ebp go into the command with the zeros)
//! Out: every register preserved
//! eax, ebx, ecx and edx = 0; command 0xB (`sub_409BF6`: the audio thread's `sub_40C4DD`
//! with those four); `sound::wait_for_commands`. Command 0xB is the music's: eax a tune,
//! ebx a second one, ecx the volume, edx what to do (2 in `sound::play_music`, 3 to
//! change the volume, 0 here). That 0 stops it is a reading of its users (before films,
//! on the way out of the front end), not of the audio thread's code. Named
//! `sound::silence` at first.
//! Seventeen callers.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::sound::queue::{COMMAND_B, Queue, send_command, sent};
use crate::sound::wait_for_commands::{Game, wait_for_commands};

// ---- Rust version ----

/// `entry` holds the registers at entry; `send` queues a command with the registers it
/// is given, `wait` waits with them.
pub fn stop_music(entry: &Regs, send: &mut dyn FnMut(u32, &Regs), wait: &mut dyn FnMut(&Regs)) {
    // Converted: `rc2k_core::sound::senders::Music::STOP`.
    let zeroed = crate::sound::senders::music_regs(entry, rc2k_core::sound::senders::Music::STOP);
    send(COMMAND_B, &zeroed);
    wait(&zeroed);
}

/// `stop_music` in the game.
pub fn stop_music_in_game(entry: &Regs) {
    stop_music(
        entry,
        &mut |command, r| unsafe { send_command(&Queue::game(), command, sent(r)) },
        &mut |r| wait_for_commands(&mut Game(sent(r))),
    );
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    stop_music_in_game(r);
    Ok(())
}
