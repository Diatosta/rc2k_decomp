//! `sound::play_timed_slot` (0x40E59E): a sound slot of the race's sounds started when
//! asked, and marked done when its time is up.
//!
//! +4 at 1 is cleared; then it starts unless it is playing (sound 0xE9 starts even
//! then): the
//! request's sound, volume (+8) and rate (+4, floats by `fild`) set, the end = the
//! clock + `sound::sample_duration`, +8 1, and `sound::play_placed_sound` (a tail jump).
//! Else, playing: +8 cleared once the clock is at or past the end (`js` on the
//! difference). 14 callers.
//! Refuses what the two leaves refuse, before writing.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::sound::sample_duration::sample_duration;
use crate::sound::sample_settings::sample_settings;
use crate::sound::sets_store::SoundRequest;

/// Converted: `rc2k_core::sound::cars::slot_turn`.
pub fn play_timed_slot(
    slot: &crate::sound::store::SoundSlot,
    request: &SoundRequest,
) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{SlotTurn, TimedSlot, from_int, slot_turn};
    let mut s = TimedSlot {
        sound: slot.sound.get(),
        start: slot.start.get(),
        playing: slot.playing.get(),
        ends: slot.ends.get(),
        volume: slot.volume.get(),
        rate: slot.rate.get(),
    };
    let clock = crate::sound::owner::sound().race.clock.last.get();
    let turn = slot_turn(&mut s, clock, sample_settings, sample_duration)?;
    slot.sound.set(s.sound);
    slot.start.set(s.start);
    slot.playing.set(s.playing);
    slot.ends.set(s.ends);
    slot.volume.set(s.volume);
    slot.rate.set(s.rate);
    match turn {
        SlotTurn::Nothing | SlotTurn::Checked => Ok(()),
        SlotTurn::Start { volume, rate, .. } => {
            request.sound.set(s.sound);
            request.volume.set(from_int(volume as i32));
            request.pitch.set(from_int(rate as i32));
            crate::sound::turn_parts::placed(request)
        }
    }
}
