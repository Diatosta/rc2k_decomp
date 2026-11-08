//! `sound::repeat_spot_voices` (0x40EB44): each voice heard and not playing told when to
//! start again: a random delay of its kind and, while its repeats last, a restart.
//!
//! For each voice heard (+4 1) and not playing: its kind's record (`unk_517FAC`, 8 bytes
//! from kind 1: words +2 the most repeats, +4 and +6 the least and most delay): +0x30 =
//! the least + `math::random_below` (most - least) (as words; no call when they are
//! equal); then with no limit (+2 0), or the repeats (+0x2C) below it (counted up), the
//! restart flag +0x24 1. One caller, `sound::spot_sounds`.
//! Refuses (before anything) a voice of kind 0 or past 0x15.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;

// ---- Rust version ----

/// Converted: `rc2k_core::sound::spot_voices::repeat`.
pub fn repeat_spot_voices(sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    use crate::sound::spot_voices::{load_voices, store_voices};
    use rc2k_core::sound::spot_voices::{Repeats, repeat};
    let table_of = sound
        .race
        .spot_picks
        .kind_repeats
        .each_ref()
        .map(|r| Repeats {
            most: r.most.get(),
            least_delay: r.least_delay.get(),
            most_delay: r.most_delay.get(),
        });
    let mut voices = load_voices(sound);
    repeat::<Refused>(&mut voices, &table_of, |_, range| {
        Ok({ crate::math::next_random::SOUNDS.below(range) }?)
    })?;
    store_voices(sound, &voices);
    Ok(())
}
