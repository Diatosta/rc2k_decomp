//! Puts a cue into a sound object's queue (a `CueQueue`: the count, entries of 0x20 bytes). Full
//! (16): nothing. Putting it first stops the playing sound by `sound::play_sound` and moves the
//! entries up one; otherwise it goes last. The entry: the cue, then its sounds (the cue's list in
//! `sets_store::table`, up to six up to a -1; one not below 0 plus the object's base), their
//! number; the count goes up by one.
//!
//! The object is a reference; the queue is walked by dword (a dword past it is refused). Refuses
//! (before anything) a cue past 0x70, the end of the 113 sound lists.

use crate::errors::{BadInput, Refused};
use crate::sound::store::SoundObjectCell;
use rc2k_core::physics::sound_queue::{CuePlacement, ENTRY_DWORDS, check_cue, place, sounds};
use rc2k_core::sound::cars::CueQueue;

/// The queue's 0x84 dwords as a slice index: the four header dwords, then the entries' eight each
/// (the first cue's place at -1 reads the entry's count: a run-on kept). One past the sixteen
/// entries is refused.
fn dword(words: &mut [u32; 0x84], i: u32) -> Result<&mut u32, BadInput> {
    words
        .get_mut(i as usize)
        .ok_or(BadInput("a dword past the cue queue"))
}

/// `cue` queued in `object`'s queue, first or last. The sound it stops on the way (a queue begun
/// again) goes as a volume of 0, of which the audio thread reads only the sound.
// Core: `rc2k_core::physics::sound_queue::{check_cue, place, sounds}`.
pub fn queue_sound_cue(
    first: bool,
    cue: u32,
    object: &SoundObjectCell,
    audio: &crate::audio::owner::Audio,
    cue_lists: &crate::sound::sets_store::CueLists,
) -> Result<(), Refused> {
    check_cue(cue)?;
    // The queue is worked on as the game's dwords, so that a dword past it is refused as before.
    let mut w = object.borrow().cue_queue.to_words();
    let sound_offset = object.borrow().sound_offset;
    let count = w[3];
    // The entry's first dword (after the header, n entries of 0x20 bytes each).
    let mut entry = 4u32;
    match place(count, first) {
        CuePlacement::Full => return Ok(()),
        CuePlacement::At(n) => entry = entry.wrapping_add(n.wrapping_mul(ENTRY_DWORDS)),
        CuePlacement::First { count } => {
            // The playing sound: the first cue's sound at its place.
            let at = w[1].wrapping_add(6);
            let playing = *dword(&mut w, at)?;
            crate::sound::turn_parts::play(audio, playing, 0, 0, [0; 3]);
            w[1] = u32::MAX;
            // The entries moved up one, from the last dword down.
            let n = count.wrapping_mul(ENTRY_DWORDS);
            for k in 0..n {
                let at = entry + n - 1 - k;
                let from = *dword(&mut w, at)?;
                *dword(&mut w, at + ENTRY_DWORDS)? = from;
            }
        }
    }
    w[3] = w[3].wrapping_add(1);
    *dword(&mut w, entry)? = cue;
    let list = cue_lists.list(cue)?;
    let failed = std::cell::Cell::new(None);
    let (written, n) = sounds(
        |k| {
            list.get(k as usize).unwrap_or_else(|e| {
                failed.set(Some(e));
                0
            })
        },
        sound_offset,
    );
    if let Some(e) = failed.take() {
        return Err(e.into());
    }
    for (k, &v) in written.iter().enumerate() {
        *dword(&mut w, entry + 2 + k as u32)? = v;
    }
    *dword(&mut w, entry + 1)? = n;
    object.borrow_mut().cue_queue = CueQueue::from_words(&w);
    Ok(())
}
