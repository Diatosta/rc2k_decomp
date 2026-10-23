//! `sound::drop_cue` (0x40EE32): one cue taken out of a sound object's queue.
//!
//! In:  eax the cue's place, edi the queue (the object +0x298: +0xC the count, cues of
//!      0x20 bytes from +0x10; `physics::queue_sound_cue`)
//! Out: cues moved: eax the place x 0x20, ecx 0; else ecx the count (less one when it
//!      dropped); everything else preserved
//! Nothing for a place at or past the count; else the count goes down by one and the
//! cues after the place move up (`rep movsd`). One caller, `sound::play_cue_queue`
//! (place 0: the cue played out).
//! Refuses (before anything) a null queue and a count over 16 (the move would run past
//! the object).
//! Shadow mode: both, the queue compared.
//! Self-test: the original against the Rust version on a test queue.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};

// ---- Rust version ----

/// Converted: `rc2k_core::sound::cars::drop_cue`. The cue at `place` dropped from the
/// queue at `queue`.
pub fn drop_cue(queue: u32, place: u32) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{CueQueue, drop_cue};
    let q = queue as *mut [u32; 0x84];
    if q.is_null() {
        return Err(BadInput("no queue").into());
    }
    let mut cues = CueQueue::from_words(&unsafe { q.read_unaligned() });
    drop_cue(&mut cues, place).map_err(|e| BadInput(e.0))?;
    unsafe { q.write_unaligned(cues.to_words()) };
    Ok(())
}
