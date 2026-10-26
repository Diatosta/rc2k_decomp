//! `drive::countdown_sound` (0x40EC9C): for a listed car (eax 0 or 1) the count down's
//! sound ebx played on its channel (`sound::play_sound`: eax + ebx, ebx and edx -0x6F,
//! ecx, edi, ebp 0). All in `pusha` / `popa`.
//!
//! In:  eax the car's sound slot, ebx the sound
//! Out: every register kept; no flags read
//!
//! Caller: `drive::countdown_cue`.

/// The Rust version.
// Converted: `rc2k_core::drive::countdown::beep_sound`.
pub fn countdown_sound(slot: u32, sound: u32) {
    let Some(beep) = rc2k_core::drive::countdown::beep_sound(slot, sound) else {
        return;
    };
    crate::sound::play_sound::play_sound(beep.sound, beep.volume, beep.volume, [0; 3]);
}
