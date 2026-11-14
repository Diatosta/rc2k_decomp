//! `sound::thin_spot_voices_all`: `sound::thin_spot_voices` for the twelve sounds of which only one
//! voice is heard: 0x1B1 .. 0x1B6, 0x1B9, 0x1BA, 0x1B0, 0x1BE, 0x1C0, 0x1C3.
//!
//! Each call is for one of those sounds. One caller,
//! `sound::spot_sounds`.

use crate::errors::Refused;

/// `sound::thin_spot_voices` for each sound thinned.
pub fn thin_spot_voices_all(
    owner: &crate::sound::owner::Sound,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    // The logic is `rc2k_core::sound::spot_voices::THINNED_SOUNDS`.
    for sound in rc2k_core::sound::spot_voices::THINNED_SOUNDS {
        crate::sound::thin_spot_voices::thin_spot_voices(sound, owner, system);
    }
    Ok(())
}
