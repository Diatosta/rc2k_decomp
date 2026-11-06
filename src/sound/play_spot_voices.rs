//! `sound::play_spot_voices` (0x40EB9E): each voice heard this turn sent: started when
//! told to, else kept going while it plays, and stopped once its time is up.
//!
//! In:  esi the request
//! Out: every register preserved (`pusha` / `popa`)
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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87::from_int;
use crate::sound::sample_duration::sample_duration;
use crate::sound::sample_settings::{OWN, RECORDS, sample_settings};
use crate::sound::sets_store::SoundRequest;
use crate::sound::spot_voices::{VOICE_COUNT, set, voice, vw};
use rc2k_core::sound::spot_voices::{Played, REQUEST_FROM, end_time, going_on, played, time_up};

// ---- Rust version ----

/// The spots' voices played through `request`.
pub fn play_spot_voices(request: &SoundRequest) -> Result<(), Refused> {
    let clock = crate::sound::owner::sound().race.clock.last.get();
    for n in 0..VOICE_COUNT {
        let v = voice(n);
        if vw(v, 4)? != 1 {
            continue;
        }
        set(v, 4, 0)?;
        let sound = vw(v, 0x14)?;
        request.sound.set(sound);
        // Converted: `rc2k_core::sound::spot_voices` (`played`, `end_time`,
        // `going_on`, `REQUEST_FROM`, `time_up`).
        let failed = std::cell::Cell::new(None);
        let kind = played(vw(v, 0x24)?, || {
            vw(v, 0x28).unwrap_or_else(|e| {
                failed.set(Some(e));
                0
            })
        });
        if let Some(e) = failed.take() {
            return Err(e.into());
        }
        match kind {
            Played::Start => {
                set(v, 0x24, 0)?;
                let (volume, rate) = sample_settings(sound, OWN, OWN)?;
                set(v, 0x3C, from_int(volume as i32))?;
                set(v, 0x38, from_int(rate as i32))?;
                set(
                    v,
                    0x34,
                    end_time(sample_duration(sound, rate)?, clock, vw(v, 0x30)?),
                )?;
                set(v, 0x28, 1)?;
            }
            Played::Silent => continue,
            Played::GoOn => {
                if sound >= RECORDS {
                    return Err(BadInput("a sound past the sample records").into());
                }
                let looping = crate::audio::store::SAMPLE_RECORDS.records[sound as usize]
                    .get()
                    .voice_kind;
                request.sound.set(going_on(request.sound.get(), looping));
            }
        }
        for (to, from) in REQUEST_FROM {
            request
                .word(to)
                .ok_or(BadInput("a request word past its ten"))?
                .set(vw(v, from)?);
        }
        crate::sound::turn_parts::placed(request)?;
        if time_up(clock, vw(v, 0x34)?) {
            request.sound.set(vw(v, 0x14)?);
            crate::sound::turn_parts::stop(request)?;
            set(v, 0x28, 0)?;
        }
    }
    Ok(())
}
