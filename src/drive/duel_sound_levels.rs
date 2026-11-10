//! `drive::duel_sound_levels` (0x45C596): two sound levels from the first car block
//! (`g_carBlockStore`): +0x900 / 4 - 4, held 0 ..= 0x40 (by the signs), into `dword_5165BC`; at
//! 0x38 or more `dword_5165A8` 1 when a random number (`math::next_random`) x 500 has a high dword
//! of 0 (a rare draw); (+0x908 squared, low dword) >> 12 - 8, held, into `dword_5165C0`.
//!
//! Caller: `drive::replay_step`.

use crate::race::blocks_store::CAR_BLOCKS;
use rc2k_core::drive::replay_views::{RARE_FROM, first_level, rare_draw, second_level};

// Converted: `rc2k_core::drive::replay_views::{first_level, rare_draw, second_level}`.
pub fn duel_sound_levels(sound: &crate::sound::owner::Sound) {
    let stage = &sound.race.stage;
    stage
        .loop_volume_a
        .set(first_level(CAR_BLOCKS.cars[0].surface_noise_a.bits()));
    if stage.loop_volume_a.get() >= RARE_FROM {
        let r = crate::math::next_random::SOUNDS.next();
        if rare_draw(r) {
            stage.timed_arm.set(1);
        }
    }
    let (level, _) = second_level(CAR_BLOCKS.cars[0].surface_noise_second.bits());
    stage.loop_volume_b.set(level);
}
