//! For kinds with a `kind_hit_sounds` entry (below 0x22), the same four tests as
//! `physics::drop_hit_parts` on the hit timers and the bits 1 .. 4 of `parts_present`, by
//! `physics::sound_on_bit` (the timer 256).
//!
//! Refuses what `sound_on_bit` refuses.

use crate::errors::Refused;
use crate::math::x87;
use crate::physics::drop_hit_parts::hit_pick;
use crate::physics::sound_on_bit::sound_on_bit;
use crate::race::kind_store::KindTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::scene::matrix::Matrix;
use rc2k_core::physics::parts::{SOUND_TESTS, has_hit_sounds};

// Core: `rc2k_core::physics::parts::{has_hit_sounds, hit_tests, SOUND_TESTS}`.
pub fn sound_hit_parts(
    numbers: &Numbers,
    view_turn: &Matrix,
    shared: &RaceShared,
    kind_tables: &KindTables,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    if !has_hit_sounds(shared.car().car_kind, |kind| {
        kind_tables.kind_hit_sounds[kind as usize]
    }) {
        return Ok(());
    }
    let (noise_word, parts_present, timers) = {
        let car = shared.car();
        (car.noise_word, car.parts_present, car.tyre_wears)
    };
    if let Some(k) = hit_pick(noise_word, parts_present, numbers, &SOUND_TESTS, &timers) {
        { sound_on_bit(numbers, view_turn, shared, SOUND_TESTS[k].1 & 0xFF, sound) }?;
        shared.car_mut().tyre_wears[k] = x87::loaded(numbers.k256);
    }
    Ok(())
}
