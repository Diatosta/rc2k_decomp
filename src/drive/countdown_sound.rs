//! `drive::countdown_sound` (0x40EC9C): for a listed car (eax 0 or 1) the count down's
//! sound ebx played on its channel (`sound::play_sound`: eax + ebx, ebx and edx -0x6F,
//! ecx, edi, ebp 0). All in `pusha` / `popa`.
//!
//! In:  eax the car's sound slot, ebx the sound
//! Out: every register kept; no flags read
//!
//! Caller: `drive::countdown_cue`.
//! Shadow mode runs the original alone (a sound command).
//! Self-test: none (a sound command; `sound::play_sound`'s own test covers it).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

/// The Rust version.
// Converted: `rc2k_core::drive::countdown::beep_sound`.
pub fn countdown_sound(slot: u32, sound: u32) {
    let Some(beep) = rc2k_core::drive::countdown::beep_sound(slot, sound) else {
        return;
    };
    crate::sound::play_sound::play_sound(beep.sound, beep.volume, beep.volume, [0; 3]);
}
