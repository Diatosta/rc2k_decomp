//! `sound::merge_spot_voices_all` (0x40E7CA): `sound::merge_spot_voices` for the six
//! sounds whose voices are merged into one: 0x1B7, 0x1B8, 0x1BB, 0x1BC, 0x1C1, 0x1BF.
//!
//! Each call is for one of those sounds. One caller,
//! `sound::spot_sounds`.

use crate::errors::Refused;

/// `sound::merge_spot_voices` for each sound merged.
pub fn merge_spot_voices_all(owner: &crate::sound::owner::Sound) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::spot_voices::MERGED_SOUNDS`.
    for sound in rc2k_core::sound::spot_voices::MERGED_SOUNDS {
        crate::sound::merge_spot_voices::merge_spot_voices(sound, owner);
    }
    Ok(())
}
