//! `sound::play_cue_queue` (0x40ED4A): a car's queue of spoken cues played, one sound
//! after the other.
//!
//! Takes: the car's sound object
//! Only for an object in use (+8 is 1) with cues queued (+0x298 +0xC, the queue of
//! `physics::queue_sound_cue`): the queue's +4 is the place in the first cue (-1: none
//! begun), +8 the time the next sound is due. Unless none is begun, nothing until then
//! (`js` on the clock's difference). Then the place moves on; past the cue's sounds (its
//! +4) the cue is dropped (`sound::drop_cue`, place 0) and the next begun at 0, or with
//! none left the place is -1. Else its sound: due again after
//! `sound::sample_duration` (the sample's own rate) + 25 ms (sounds 0x13F and 0x140:
//! 0x1000000 later, so never), and `sound::play_sound` with the cue's volume, the
//! sample's own rate and no place.
//! Two callers, `sound::play_cue_queues`.
//! Refuses a null object, a queue of more than 16 or a cue of more than six sounds
//! (before anything), and the duration's refusals (what was written put back).

use crate::errors::Refused;
use crate::sound::sample_duration::sample_duration;

/// Converted: `rc2k_core::sound::cars::cue_turn`; the drops through `sound::drop_cue`, the
/// queue in the game's memory meanwhile. Object `so`'s queue of cues: the next one played
/// when it is due.
pub fn play_cue_queue(
    so: &crate::sound::store::SoundObject,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{CueQueue, CueTurn, OWN, cue_turn};
    let in_use = so.in_use.get() == 1;
    let q = &so.cue_queue;
    let mut queue = CueQueue::from_words(&crate::sound::drop_cue::read_words(q)?);
    let clock = sound.race.clock.get().last.get();
    if !in_use {
        return Ok(());
    }
    let turn = cue_turn::<Refused>(
        &mut queue,
        in_use,
        clock,
        |sound| Ok(sample_duration(audio, sound, OWN)?),
        |queue, _| {
            // The first cue taken out (`drop_cue` at place 0).
            crate::sound::drop_cue::write_words(q, queue.to_words())?;
            crate::sound::drop_cue::drop_cue(q, 0)?;
            *queue = CueQueue::from_words(&crate::sound::drop_cue::read_words(q)?);
            Ok(())
        },
    )?;
    match turn {
        CueTurn::NotInUse | CueTurn::Empty | CueTurn::NotDue => return Ok(()),
        _ => {}
    }
    crate::sound::drop_cue::write_words(q, queue.to_words())?;
    if let CueTurn::Play { sound } = turn {
        crate::sound::play_sound::play_sound_in(audio, sound, OWN, OWN, [0; 3]);
    }
    Ok(())
}
