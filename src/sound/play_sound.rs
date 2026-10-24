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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::audio::store::Slot;
use crate::sound::queue::{Queue, send_command};
use crate::store::game_read;

// ---- game data ----

const HELD_BACK_RVA: Slot = Slot::at(0x506C84); // dword_506C84
const MODE_RVA: Slot = Slot::at(0x4F82BC); // dword_4F82BC
const PLAY_1: u32 = 1;

// ---- Rust version ----

/// `sound` at `volume` and `rate`; `place` its ecx, edi and ebp (the pan, or in the 3D
/// modes the place). The audio thread reads every register of command 1.
pub fn play_sound(sound: u32, volume: u32, rate: u32, place: [u32; 3]) {
    let (held_back, mode) = unsafe { (game_read(HELD_BACK_RVA), game_read(MODE_RVA)) };
    // Converted: `rc2k_core::sound::senders::sound_place`.
    if let Some([esi, ecx, edi, ebp]) =
        rc2k_core::sound::senders::sound_place(held_back, mode, place)
    {
        let registers = [sound, volume, ecx, rate, esi, edi, ebp];
        unsafe { send_command(&Queue::game(), PLAY_1, registers) };
    }
}
