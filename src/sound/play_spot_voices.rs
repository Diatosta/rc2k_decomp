//! `sound::play_spot_voices`: each voice heard this turn sent: started when told to, else kept
//! going while it plays, and stopped once its time is up.
//!
//! Takes the request. For each voice heard (the flag is cleared): the request's sound is the
//! voice's. Told to start (the restart flag is cleared): its volume and rate
//! (`sound::sample_settings`, the sample's own) become the voice's floats, the end is
//! `sound::sample_duration` plus the clock plus its delay, and it is playing. Else only a playing
//! one goes on, its sound with bit 31 set unless the sample loops. Then the request gets the
//! volume, the rate, the pitch and volume factors and the place, and `sound::play_placed_sound`
//! plays it; once the clock is at or past the end, the sound is stopped (`sound::stop_sound`) and
//! the voice is no longer playing. A sound past the sample records and the two leaves' refusals put
//! back what was written (sounds already sent stay sent).

use crate::errors::{BadInput, Refused};
use crate::math::x87::from_int;
use crate::sound::sample_duration::sample_duration;
use crate::sound::sample_settings::{OWN, RECORDS, sample_settings};
use crate::sound::sets_store::SoundRequest;
use crate::sound::spot_voices::VOICE_COUNT;
use rc2k_core::sound::spot_voices::{Played, end_time, going_on, played, time_up};

/// The spots' voices played through `request`.
pub fn play_spot_voices(
    request: &mut SoundRequest,
    audio: &crate::audio::owner::Audio,
    owner: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let clock = owner.race.clock.get().borrow().last;
    let voices = &owner.race.spot_voices.get().voices;
    for n in 0..VOICE_COUNT as usize {
        // A copy of the voice, written back at each step the original writes it at (a refusal
        // leaves the steps before it done).
        let mut v = voices.borrow()[n];
        if v.heard != 1 {
            continue;
        }
        v.heard = 0;
        voices.borrow_mut()[n].heard = 0;
        let sound = v.sound;
        request.sound = sound;
        // The logic is `rc2k_core::sound::spot_voices` (`played`, `end_time`, `going_on`,
        // `time_up`).
        let kind = played(v.restart, || v.playing);
        match kind {
            Played::Start => {
                v.restart = 0;
                voices.borrow_mut()[n].restart = 0;
                let (volume, rate) = sample_settings(audio, sound, OWN, OWN)?;
                v.volume = from_int(volume as i32);
                v.rate = from_int(rate as i32);
                voices.borrow_mut()[n] = v;
                v.ends = end_time(sample_duration(audio, sound, rate)?, clock, v.delay);
                v.playing = 1;
                voices.borrow_mut()[n] = v;
            }
            Played::Silent => continue,
            Played::GoOn => {
                if sound >= RECORDS {
                    return Err(BadInput("a sound past the sample records").into());
                }
                let looping =
                    crate::audio::sample_records::SAMPLE_CONSTANTS[sound as usize].voice_kind;
                request.sound = going_on(request.sound, looping);
            }
        }
        // The request's words the voice fills (words 2, 1, 8, 9, 3 .. 5): the volume, the rate,
        // the two factors, the place; the request holds them as dwords.
        let filled = [
            (2, v.volume.to_bits()),
            (1, v.rate.to_bits()),
            (8, v.pitch_factor.to_bits()),
            (9, v.volume_factor.to_bits()),
            (3, v.place[0].to_bits()),
            (4, v.place[1].to_bits()),
            (5, v.place[2].to_bits()),
        ];
        for (to, bits) in filled {
            request
                .set_word(to, bits)
                .ok_or(BadInput("a request word past its ten"))?;
        }
        crate::sound::turn_parts::placed(request, owner, audio)?;
        if time_up(clock, v.ends) {
            request.sound = v.sound;
            crate::sound::turn_parts::stop(request, audio)?;
            voices.borrow_mut()[n].playing = 0;
        }
    }
    Ok(())
}
