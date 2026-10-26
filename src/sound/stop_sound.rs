//! `sound::stop_sound` (0x40F1D1): plays a request's sound at volume 0.
//!
//! In:  esi = the request (its first dword is the sound; bit 0x8000: none); ecx, edx,
//!      edi, ebp go into the command as they are
//! Out: every register preserved
//! Unless the sound has bit 0x8000: `sound::play_sound` with eax the sound and ebx = 0.
//! That it stops the sound is a reading (a volume of 0).
//! 14 callers (the race's sounds).
//! Refuses a null request.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::BadInput;
use crate::pointers::GamePtr;

// ---- Rust version ----

/// The request's sound stopped (`sound::play_sound` with a volume of 0, of which the
/// audio thread reads only the sound and the volume) unless it has bit 0x8000.
pub fn stop_sound(request: u32) -> Result<(), BadInput> {
    if request == 0 {
        return Err(BadInput("no request"));
    }
    let sound: u32 = GamePtr::new(request).read("a sound request")?;
    // Converted: `rc2k_core::sound::senders::stopped`.
    if let Some(sound) = rc2k_core::sound::senders::stopped(sound) {
        crate::sound::play_sound::play_sound(sound, 0, 0, [0; 3]);
    }
    Ok(())
}
