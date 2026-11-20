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
//! 0x1AF. Refuses nothing of its own (what the slot refuses puts back what was written; sounds
//! already sent stay sent).

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
    let mut request = crate::sound::turn_parts::request(sound);
    let stages = sound.race.stage.get();
    let timed_slot = sound.race.timed_slot.get();
    request.pitch_factor = 1.0;
    request.volume_factor = 1.0;
    for (n, sound) in ONE_OFFS {
        let flagged = {
            let mut stage = stages.borrow_mut();
            let flagged = stage.once[n] == 1;
            if flagged {
                stage.once[n] = 0;
            }
            flagged
        };
        if flagged {
            play(sound, OWN, OWN, [0, 0, MINUS_ONE]);
        }
    }
    // The logic is `rc2k_core::sound::stage::wait_turn`.
    let clock = sound.race.clock.get().borrow().last;
    let (slot, place) = {
        let mut stage = stages.borrow_mut();
        let mut wait = rc2k_core::sound::stage::Wait {
            arm: stage.timed_arm,
            delay: stage.timed_delay,
            due: stage.timed_due,
            waiting: stage.timed_waiting,
            start: timed_slot.borrow().start,
        };
        let slot = rc2k_core::sound::stage::wait_turn(&mut wait, clock);
        stage.timed_arm = wait.arm;
        stage.timed_due = wait.due;
        stage.timed_waiting = wait.waiting;
        if wait.start != timed_slot.borrow().start {
            timed_slot.borrow_mut().start = wait.start;
        }
        (slot, stage.timed_place)
    };
    if slot {
        request.place = place;
        crate::sound::play_timed_slot::play_timed_slot(timed_slot, &mut request, sound, audio)?;
    }
    let (volume_a, volume_b) = {
        let stage = stages.borrow();
        (stage.loop_volume_a, stage.loop_volume_b)
    };
    play(
        0x1C5,
        ((volume_a as i32) >> 1) as u32,
        OWN,
        [0, ONE, MINUS_ONE],
    );
    play(0x1C6, volume_b, OWN, [0, ONE, 0]);
    for (n, sound) in RANDOMS {
        {
            let mut stage = stages.borrow_mut();
            if stage.random[n] != 1 {
                continue;
            }
            stage.random[n] = 0;
        }
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
