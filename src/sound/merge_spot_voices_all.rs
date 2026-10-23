//! `sound::merge_spot_voices_all` (0x40E7CA): `sound::merge_spot_voices` for the six
//! sounds whose voices are merged into one: 0x1B7, 0x1B8, 0x1BB, 0x1BC, 0x1C1, 0x1BF.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Each call with eax the sound and the other registers the one before left. One caller,
//! `sound::spot_sounds`.
//! Shadow mode: both (nothing sent), the voices and variables compared.
//! Self-test: a copy whose calls go to a stand-in, against the Rust version.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;

// ---- Rust version ----

/// `sound::merge_spot_voices` for each sound merged.
pub fn merge_spot_voices_all() -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::spot_voices::MERGED_SOUNDS`.
    for sound in rc2k_core::sound::spot_voices::MERGED_SOUNDS {
        crate::sound::merge_spot_voices::merge_spot_voices(sound)?;
    }
    Ok(())
}
