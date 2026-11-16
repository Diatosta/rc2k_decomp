//! `sound::drop_cue`: one cue taken out of a sound object's queue.
//!
//! Nothing for a place at or past the count; else the count goes down by one and the cues after the
//! place move up. Refuses (before anything) a null queue and a count over 16 (the move would run
//! past the object).

use crate::errors::{BadInput, Refused};

/// The logic is `rc2k_core::sound::cars::drop_cue`. The cue at `place` dropped from the queue at
/// `queue`.
pub fn drop_cue(q: &crate::sound::store::CueQueue, place: u32) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{CueQueue, drop_cue};
    let mut cues = CueQueue::from_words(&read_words(q)?);
    drop_cue(&mut cues, place).map_err(|e| BadInput(e.0))?;
    write_words(q, cues.to_words())?;
    Ok(())
}

/// The queue's 0x84 dwords: its four header dwords, then the cues' eight each.
pub fn read_words(
    q: &crate::sound::store::CueQueue,
) -> Result<[u32; 0x84], crate::errors::BadInput> {
    let head = [&q.first, &q.place, &q.due, &q.count];
    let cues = q.cues.iter().flat_map(|cue| {
        [&cue.first, &cue.count]
            .into_iter()
            .chain(cue.sounds.iter())
    });
    let mut words = head
        .into_iter()
        .chain(cues)
        .map(crate::memory::session::PlainDword::get);
    Ok(std::array::from_fn(|_| words.next().unwrap_or(0)))
}

/// `words` as the queue's dwords.
pub fn write_words(
    q: &crate::sound::store::CueQueue,
    words: [u32; 0x84],
) -> Result<(), crate::errors::BadInput> {
    let head = [&q.first, &q.place, &q.due, &q.count];
    let cues = q.cues.iter().flat_map(|cue| {
        [&cue.first, &cue.count]
            .into_iter()
            .chain(cue.sounds.iter())
    });
    for (cell, word) in head.into_iter().chain(cues).zip(words) {
        cell.set(word);
    }
    Ok(())
}
