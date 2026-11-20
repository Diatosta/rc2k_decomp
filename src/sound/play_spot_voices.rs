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
use crate::sound::spot_voices::{VOICE_COUNT, voice};
use rc2k_core::sound::spot_voices::{Played, end_time, going_on, played, time_up};

/// The spots' voices played through `request`.
pub fn play_spot_voices(
    request: &SoundRequest,
    audio: &crate::audio::owner::Audio,
    owner: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let clock = owner.race.clock.get().borrow().last;
    for n in 0..VOICE_COUNT {
        let v = voice(n, owner);
        if v.heard.get() != 1 {
            continue;
        }
        v.heard.set(0);
        let sound = v.sound.get();
        request.sound.set(sound);
        // The logic is `rc2k_core::sound::spot_voices` (`played`, `end_time`, `going_on`,
        // `time_up`).
        let kind = played(v.restart.get(), || v.playing.get());
        match kind {
            Played::Start => {
                v.restart.set(0);
                let (volume, rate) = sample_settings(audio, sound, OWN, OWN)?;
                v.volume.set(from_int(volume as i32));
                v.rate.set(from_int(rate as i32));
                v.ends.set(end_time(
                    sample_duration(audio, sound, rate)?,
                    clock,
                    v.delay.get(),
                ));
                v.playing.set(1);
            }
            Played::Silent => continue,
            Played::GoOn => {
                if sound >= RECORDS {
                    return Err(BadInput("a sound past the sample records").into());
                }
                let looping =
                    crate::audio::sample_records::SAMPLE_CONSTANTS[sound as usize].voice_kind;
                request.sound.set(going_on(request.sound.get(), looping));
            }
        }
        // The request's words the voice fills (words 2, 1, 8, 9, 3 .. 5): the volume, the rate,
        // the two factors, the place; the request holds them as dwords.
        let filled = [
            (2, v.volume.bits()),
            (1, v.rate.bits()),
            (8, v.pitch_factor.bits()),
            (9, v.volume_factor.bits()),
            (3, v.place[0].bits()),
            (4, v.place[1].bits()),
            (5, v.place[2].bits()),
        ];
        for (to, bits) in filled {
            request
                .set_word(to, bits)
                .ok_or(BadInput("a request word past its ten"))?;
        }
        crate::sound::turn_parts::placed(request, owner, audio)?;
        if time_up(clock, v.ends.get()) {
            request.sound.set(v.sound.get());
            crate::sound::turn_parts::stop(request, audio)?;
            v.playing.set(0);
        }
    }
    Ok(())
}
