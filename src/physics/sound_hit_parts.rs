//! For kinds with a `kind_hit_sounds` entry (below 0x22), the same four tests as
//! `physics::drop_hit_parts` on the hit timers and the bits 1 .. 4 of `parts_present`, by
//! `physics::sound_on_bit` (the timer 256).
//!
//! Refuses what `sound_on_bit` refuses.

use crate::errors::Refused;
use crate::physics::drop_hit_parts::hit_tests;
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
    if !has_hit_sounds(shared.car().car_kind.get(), |kind| {
        kind_tables.kind_hit_sounds[kind as usize]
    }) {
        return Ok(());
    }
    hit_tests(
        shared.car(),
        numbers,
        &SOUND_TESTS,
        &shared.car().tyre_wears,
        &mut |bit| {
            { sound_on_bit(numbers, view_turn, shared, bit & 0xFF, sound) }?;
            Ok(())
        },
    )
}
