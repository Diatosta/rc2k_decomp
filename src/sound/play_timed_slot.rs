//! `sound::play_timed_slot`: a sound slot of the race's sounds started when asked, and marked done
//! when its time is up.
//!
//! A start flag of 1 is cleared; then the slot starts unless it is playing (sound 0xE9 starts even
//! then): the request's sound, volume and rate (floats) are set, the end is the clock +
//! `sound::sample_duration`, the slot is playing, and `sound::play_placed_sound` plays it. Else, if
//! playing: the playing flag is cleared once the clock is at or past the end. Refuses what the two
//! leaves refuse, before writing.

use crate::errors::Refused;
use crate::sound::sample_duration::sample_duration;
use crate::sound::sample_settings::sample_settings;
use crate::sound::sets_store::SoundRequest;

/// The logic is `rc2k_core::sound::cars::slot_turn`.
pub fn play_timed_slot(
    slot: &impl crate::sound::store::TimedCells,
    request: &SoundRequest,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{SlotTurn, from_int, slot_turn};
    let mut s = slot.load();
    let clock = sound.race.clock.get().last.get();
    let turn = slot_turn(
        &mut s,
        clock,
        |sample, volume, rate| sample_settings(audio, sample, volume, rate),
        |sample, rate| sample_duration(audio, sample, rate),
    )?;
    slot.store(&s);
    match turn {
        SlotTurn::Nothing | SlotTurn::Checked => Ok(()),
        SlotTurn::Start { volume, rate, .. } => {
            request.sound.set(s.sound);
            request.volume.set(from_int(volume as i32));
            request.pitch.set(from_int(rate as i32));
            crate::sound::turn_parts::placed(request, sound, audio)
        }
    }
}
