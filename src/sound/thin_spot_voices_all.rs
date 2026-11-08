//! `sound::thin_spot_voices_all` (0x40E97D): `sound::thin_spot_voices` for the twelve
//! sounds of which only one voice is heard: 0x1B1 .. 0x1B6, 0x1B9, 0x1BA, 0x1B0, 0x1BE,
//! 0x1C0, 0x1C3.
//!
//! Each call with eax the sound and the other registers the one before left. One caller,
//! `sound::spot_sounds`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;

// ---- Rust version ----

/// `sound::thin_spot_voices` for each sound thinned.
pub fn thin_spot_voices_all(owner: &crate::sound::owner::Sound) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::spot_voices::THINNED_SOUNDS`.
    for sound in rc2k_core::sound::spot_voices::THINNED_SOUNDS {
        crate::sound::thin_spot_voices::thin_spot_voices(sound, owner)?;
    }
    Ok(())
}
