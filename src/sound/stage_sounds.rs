//! `sound::stage_sounds`: the stage's own sounds for the turn: three one-off sounds, a timed one,
//! two that play all the time and two at a random pitch and place.
//!
//! The request's pitch and volume factors are set to 1.0. Each of `StageSounds.once` at 1 is
//! cleared and sound 0x1AB, 0x1AC, 0x1AD played (`sound::play_sound`: the sound's own volume and
//! rate, pan 0, a third place value of -1.0). `StageSounds.timed_arm` at 1 (cleared) arms the timed
//! sound: due (`StageSounds.timed_due`) at the clock + `StageSounds.timed_delay`, waiting
//! (`StageSounds.timed_waiting`) 1. While it waits and is not due the slot is skipped; once due,
//! its slot (`StageSounds.timed_slot.sound`) is told to start and the wait ends. Otherwise the slot
//! (`sound::play_timed_slot`) is played at `StageSounds.timed_place`. Then sound 0x1C5 with the
//! volume `StageSounds.loop_volume_a` / 2, second place value 1.0, third -1.0, and 0x1C6 with
//! `StageSounds.loop_volume_b`, second 1.0, third 0. Each of `StageSounds.random` at 1 (cleared):
//! one `math::next_random` gives the volume (0x21 + its low five bits), the rate (the mix rate +
//! its next nine bits less 0x100 times the mix rate over 4096) and two floats of nine bits less
//! 0x100 and nine bits, x 0.01, as the first and second place values, the third 1.0: sound 0x1AE /
//! 0x1AF. One caller, `sound::race_turn`. Refuses nothing of its own (what the slot refuses puts
//! back what was written; sounds already sent stay sent).

use crate::errors::Refused;
use crate::sound::sample_settings::OWN;

/// The one-off sounds: the flag's index in `once` and the sound.
const ONE_OFFS: [(usize, u32); 3] = [(0, 0x1AB), (1, 0x1AC), (2, 0x1AD)];
/// The random sounds: the flag's index in `random` and the sound.
const RANDOMS: [(usize, u32); 2] = [(0, 0x1AE), (1, 0x1AF)];
/// 1.0 as float bits.
const ONE: u32 = 0x3F80_0000;
const MINUS_ONE: u32 = 0xBF80_0000;

/// The stage's own sounds for the turn.
pub fn stage_sounds(
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
    random: &crate::math::next_random::Randoms,
) -> Result<(), Refused> {
    let play = |sound, volume, rate, place| {
        crate::sound::play_sound::play_sound_in(audio, sound, volume, rate, place);
    };
    let request = crate::sound::turn_parts::request(sound);
    let stage = sound.race.stage.get();
    request.pitch_factor.set(1.0);
    request.volume_factor.set(1.0);
    for (n, sound) in ONE_OFFS {
        if stage.once[n].get() == 1 {
            stage.once[n].set(0);
            play(sound, OWN, OWN, [0, 0, MINUS_ONE]);
        }
    }
    // The logic is `rc2k_core::sound::stage::wait_turn`.
    let clock = sound.race.clock.get().last.get();
    let mut wait = rc2k_core::sound::stage::Wait {
        arm: stage.timed_arm.get(),
        delay: stage.timed_delay.get(),
        due: stage.timed_due.get(),
        waiting: stage.timed_waiting.get(),
        start: stage.timed_slot.start.get(),
    };
    let slot = rc2k_core::sound::stage::wait_turn(&mut wait, clock);
    stage.timed_arm.set(wait.arm);
    stage.timed_due.set(wait.due);
    stage.timed_waiting.set(wait.waiting);
    if wait.start != stage.timed_slot.start.get() {
        stage.timed_slot.start.set(wait.start);
    }
    if slot {
        for (cell, place) in request.place.iter().zip(&stage.timed_place) {
            cell.set(place.get());
        }
        crate::sound::play_timed_slot::play_timed_slot(&stage.timed_slot, request, sound, audio)?;
    }
    play(
        0x1C5,
        ((stage.loop_volume_a.get() as i32) >> 1) as u32,
        OWN,
        [0, ONE, MINUS_ONE],
    );
    play(0x1C6, stage.loop_volume_b.get(), OWN, [0, ONE, 0]);
    for (n, sound) in RANDOMS {
        if stage.random[n].get() != 1 {
            continue;
        }
        stage.random[n].set(0);
        let n = random.sounds.next();
        // The logic is `rc2k_core::sound::stage::random`.
        let r = rc2k_core::sound::stage::random(n, audio.state.mix_rate_setting.get());
        // The scratch dword holds the last float.
        system.machine().scratch.set(r.ahead.to_bits());
        play(
            sound,
            r.volume,
            r.rate,
            [r.across.to_bits(), r.ahead.to_bits(), ONE],
        );
    }
    Ok(())
}
