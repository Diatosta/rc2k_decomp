//! `sound::play_spot_voices` (0x40EB9E): each voice heard this turn sent: started when
//! told to, else kept going while it plays, and stopped once its time is up.
//!
//! Takes: the request
//! For each voice heard (+4 1, cleared): the request's sound = +0x14. Told to start
//! (+0x24 1, cleared): its volume and rate (`sound::sample_settings`, the sample's own)
//! as floats into +0x3C / +0x38, the end (+0x34) = `sound::sample_duration` plus the
//! clock plus its delay (+0x30), playing (+0x28) 1. Else only a playing one goes on, its sound
//! with bit 31 set unless the sample loops (record +0x14 is 1). Then the request: +8
//! the volume, +4 the rate, +0x20 / +0x24 the factors (+0x48 / +0x4C), the place, and
//! `sound::play_placed_sound`; once the clock is at or past the end (`js`), the sound
//! stopped (`sound::stop_sound`) and playing 0. One caller, `sound::spot_sounds`.
//! A sound past the sample records and the
//! two leaves' refusals put back what was written (sounds already sent stay sent).

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
    let clock = owner.race.clock.get().last.get();
    for n in 0..VOICE_COUNT {
        let v = voice(n, owner);
        if v.heard.get() != 1 {
            continue;
        }
        v.heard.set(0);
        let sound = v.sound.get();
        request.sound.set(sound);
        // Converted: `rc2k_core::sound::spot_voices` (`played`, `end_time`,
        // `going_on`, `time_up`).
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
                let looping = crate::audio::store::SAMPLE_CONSTANTS[sound as usize].voice_kind;
                request.sound.set(going_on(request.sound.get(), looping));
            }
        }
        // The request's dwords the voice fills (request words 2, 1, 8, 9, 3 .. 5): the volume, the rate,
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
