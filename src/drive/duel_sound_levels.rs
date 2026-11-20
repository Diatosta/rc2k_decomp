//! Two sound levels from the first car block: `surface_noise_a` / 4 - 4, held 0 ..= 0x40 (by the signs), into
//! `StageSounds.loop_volume_a`; at 0x38 or more `StageSounds.timed_arm` 1 when a random number
//! (`math::next_random`) x 500 has a high dword of 0 (a rare draw); (`surface_noise_second` squared, low dword) >>
//! 12 - 8, held, into `StageSounds.loop_volume_b`.

use crate::race::owner::Race;
use rc2k_core::drive::replay_views::{RARE_FROM, first_level, rare_draw, second_level};

pub(super) fn duel_sound_levels(race: &Race, sound: &crate::sound::owner::Sound) {
    let stage = sound.race.stage.get();
    stage
        .loop_volume_a
        .set(first_level(race.car_blocks().cars[0].surface_noise_a.get()));
    if stage.loop_volume_a.get() >= RARE_FROM {
        let r = race.shared.random.sounds.next();
        if rare_draw(r) {
            stage.timed_arm.set(1);
        }
    }
    let (level, _) = second_level(race.car_blocks().cars[0].surface_noise_second.get());
    stage.loop_volume_b.set(level);
}
