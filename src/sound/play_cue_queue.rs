//! `sound::play_cue_queue` (0x40ED4A): a car's queue of spoken cues played, one sound
//! after the other.
//!
//! In:  ebx the car's sound object
//! Out: (from a `play_sound`) eax the sound, ebx and edx -0x6F, ecx, edi, ebp 0; (the
//!      queue run out) eax 0, ebx the place, ecx `drop_cue`'s, edi the queue; (not due)
//!      eax the clock, edi the queue; esi preserved
//! Only for an object in use (+8 is 1) with cues queued (+0x298 +0xC, the queue of
//! `physics::queue_sound_cue`): the queue's +4 is the place in the first cue (-1: none
//! begun), +8 the time the next sound is due. Unless none is begun, nothing until then
//! (`js` on the clock's difference). Then the place moves on; past the cue's sounds (its
//! +4) the cue is dropped (`sound::drop_cue`, place 0) and the next begun at 0, or with
//! none left the place is -1. Else its sound: due again after
//! `sound::sample_duration` (the sample's own rate) + 25 ms (sounds 0x13F and 0x140:
//! 0x1000000 later, so never: a reading), and `sound::play_sound` with ebx (the volume)
//! and edx -0x6F, ecx, edi, ebp 0.
//! Two callers, `sound::play_cue_queues`.
//! Refuses a null object, a queue of more than 16 or a cue of more than six sounds
//! (before anything), and the duration's refusals (what was written put back).
//! A near `jnz` in its first five bytes: private copy (0xA4 bytes, the three calls
//! relocated), manual patch.
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls (`drop_cue`, which the stand-in does, `play_sound`) go
//! to stand-ins (the one of
//! `sample_duration` left), against the Rust version, on a test object.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::physics::flow::guard;
use crate::sound::sample_duration::sample_duration;
use crate::sound::turn_parts::{CLOCK, OBJECT_LEN, PLAY, Part, turn_entry};

const DROP: u32 = 0x40EE32;
const QUEUE: u32 = 0x298;

// ---- Rust version ----

/// Converted: `rc2k_core::sound::cars::cue_turn`; the drops through `sound::drop_cue` with
/// the registers the original has there, the queue in the game's memory meanwhile.
pub fn play_cue_queue_in(c: &mut Regs, part: Part) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{CueQueue, CueTurn, OWN, cue_turn};
    let o = c.ebx;
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    let in_use = unsafe { ((o + 8) as *const u32).read_unaligned() } == 1;
    let q = (o + QUEUE) as *mut [u32; 0x84];
    let mut queue = CueQueue::from_words(&unsafe { q.read_unaligned() });
    let clock: u32 = unsafe { game_read(rva(CLOCK)) };
    if !in_use {
        return Ok(());
    }
    c.edi = q as u32;
    guard(&[(o, OBJECT_LEN)], || {
        let turn = cue_turn::<Refused>(
            &mut queue,
            in_use,
            clock,
            |sound| Ok(sample_duration(sound, OWN)?),
            |queue, place| {
                unsafe { q.write_unaligned(queue.to_words()) };
                (c.eax, c.ebx) = (0, place);
                part(DROP, c)?;
                *queue = CueQueue::from_words(&unsafe { q.read_unaligned() });
                Ok(())
            },
        )?;
        match turn {
            CueTurn::NotInUse | CueTurn::Empty => return Ok(()),
            CueTurn::NotDue => {
                c.eax = clock;
                return Ok(());
            }
            _ => {}
        }
        unsafe { q.write_unaligned(queue.to_words()) };
        match turn {
            CueTurn::Play { sound } => {
                (c.eax, c.ebx, c.edx, c.ecx, c.edi, c.ebp) = (sound, OWN, OWN, 0, 0, 0);
                part(PLAY, c)
            }
            _ => Ok(()),
        }
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, play_cue_queue_in)
}
