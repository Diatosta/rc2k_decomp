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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::sound::turn_parts::Part;

// ---- Rust version ----

/// Converted: `rc2k_core::sound::cars::drop_cue`.
pub fn drop_cue_in(c: &mut Regs, _: Part) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{CueQueue, Dropped, drop_cue};
    let q = c.edi as *mut [u32; 0x84];
    if q.is_null() {
        return Err(BadInput("no queue").into());
    }
    let mut queue = CueQueue::from_words(&unsafe { q.read_unaligned() });
    match drop_cue(&mut queue, c.eax).map_err(|e| BadInput(e.0))? {
        Dropped::None { count } | Dropped::Last { count } => c.ecx = count,
        // `imul eax, 20h` (the cue's offset) is left in eax; `rep movsd` leaves ecx 0.
        Dropped::Moved => (c.eax, c.ecx) = (c.eax.wrapping_mul(0x20), 0),
    }
    unsafe { q.write_unaligned(queue.to_words()) };
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::sound::turn_parts::turn_entry(r, drop_cue_in)
}
