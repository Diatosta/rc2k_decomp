//! `sound::play_sound` (0x409B36): sends the audio thread a sound to play (command 1).
//!
//! In:  eax sound, ebx volume, edx; ecx, edi, ebp the sound's place when the mode is 3D
//! Out: every register preserved
//! Nothing while sounds are held back (`dword_506C84` is 1; `sub_409B6B` sets it,
//! `sub_409B76` clears it). Unless the sound mode in use (`dword_4F82BC`) has bit
//! 0x100000 (the 3D modes), the command goes with esi, edi and ebp = 0xFF800000; with
//! it, with esi = ecx and ecx = 0.
//! 14 callers. Shadow mode runs the original alone (a command must not be sent twice).
//! Self-test: a redirected copy whose two variables are the test's and whose command
//! notes its registers.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read, rva};
use crate::sound::queue::{Queue, send_command, sent};

// ---- game data ----

const HELD_BACK_RVA: usize = rva(0x506C84); // dword_506C84
const MODE_RVA: usize = rva(0x4F82BC); // dword_4F82BC
const PLAY_1: u32 = 1;

// ---- Rust version ----

/// The registers command 1 is sent with, or None if nothing is sent.
pub fn play_sound(entry: &Regs, held_back: u32, mode: u32) -> Option<Regs> {
    // Converted: `rc2k_core::sound::senders::sound_place`.
    let [esi, ecx, edi, ebp] =
        rc2k_core::sound::senders::sound_place(held_back, mode, [entry.ecx, entry.edi, entry.ebp])?;
    Some(Regs {
        esi,
        ecx,
        edi,
        ebp,
        ..*entry
    })
}

/// With the game's variables and queue: what other Rust code calls.
pub fn play_sound_in_game(entry: &Regs) {
    let (held_back, mode) = unsafe { (game_read(HELD_BACK_RVA), game_read(MODE_RVA)) };
    if let Some(with) = play_sound(entry, held_back, mode) {
        unsafe { send_command(&Queue::game(), PLAY_1, sent(&with)) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    play_sound_in_game(r);
    Ok(())
}
