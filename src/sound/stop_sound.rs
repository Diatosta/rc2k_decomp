//! `sound::stop_sound` (0x40F1D1): plays a request's sound at volume 0.
//!
//! In:  esi = the request (its first dword is the sound; bit 0x8000: none); ecx, edx,
//!      edi, ebp go into the command as they are
//! Out: every register preserved
//! Unless the sound has bit 0x8000: `sound::play_sound` with eax the sound and ebx = 0.
//! That it stops the sound is a reading (a volume of 0).
//! 14 callers (the race's sounds). Shadow mode runs the original alone.
//! Self-test: a redirected copy whose `play_sound` notes its registers.
//! Refuses a null request.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::Regs;
use crate::sound::play_sound::play_sound_in_game;

// ---- Rust version ----

/// The registers `sound::play_sound` is called with, or None if it is not called.
pub unsafe fn stop_sound(entry: &Regs) -> Result<Option<Regs>, BadInput> {
    let request = entry.esi as *const u32;
    if request.is_null() {
        return Err(BadInput("no request"));
    }
    let sound = unsafe { request.read_unaligned() };
    // Converted: `rc2k_core::sound::senders::stopped`.
    Ok(rc2k_core::sound::senders::stopped(sound).map(|sound| Regs {
        eax: sound,
        ebx: 0,
        ..*entry
    }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    match unsafe { stop_sound(r) } {
        Ok(Some(with)) => play_sound_in_game(&with),
        Ok(None) => {}
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
