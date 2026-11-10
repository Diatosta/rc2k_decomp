//! `sound::drop_cue` (0x40EE32): one cue taken out of a sound object's queue.
//!
//! Nothing for a place at or past the count; else the count goes down by one and the
//! cues after the place move up (`rep movsd`). One caller, `sound::play_cue_queue`
//! (place 0: the cue played out).
//! Refuses (before anything) a null queue and a count over 16 (the move would run past
//! the object).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};

/// Converted: `rc2k_core::sound::cars::drop_cue`. The cue at `place` dropped from the
/// queue at `queue`.
pub fn drop_cue(q: &crate::sound::store::CueQueue, place: u32) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{CueQueue, drop_cue};
    let mut cues = CueQueue::from_words(&read_words(q)?);
    drop_cue(&mut cues, place).map_err(|e| BadInput(e.0))?;
    write_words(q, cues.to_words())?;
    Ok(())
}

/// The queue's 0x84 dwords.
pub fn read_words(
    q: &crate::sound::store::CueQueue,
) -> Result<[u32; 0x84], crate::errors::BadInput> {
    let mut bytes = [0u8; 0x84 * 4];
    crate::pointers::load_cells(q, 0, &mut bytes)?;
    Ok(std::array::from_fn(|n| {
        u32::from_le_bytes(bytes[4 * n..4 * n + 4].try_into().unwrap())
    }))
}

/// `words` as the queue's dwords.
pub fn write_words(
    q: &crate::sound::store::CueQueue,
    words: [u32; 0x84],
) -> Result<(), crate::errors::BadInput> {
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    crate::pointers::store_cells(q, 0, &bytes)
}
