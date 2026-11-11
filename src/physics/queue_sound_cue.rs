//! `physics::queue_sound_cue` (0x40EE5C): a cue goes into a sound object's queue (the object's
//! +0x298; +0xC the count, entries of 0x20 bytes from +0x10). Full (16): nothing. Putting it first
//! stops the playing sound (+0x18 [+4]) by `sound::play_sound` and moves the entries up one;
//! otherwise it goes last. The entry: the cue, then its sounds (the cue's list in
//! `sets_store::table`, up to six up to a -1; one not below 0 plus the object's base) at +8 .., their
//! number at +4; the count goes up by one.
//!
//! Caller: `physics::cue_car_sound`. The object is a reference; the queue is its typed
//! +0x298 (`CueQueue`), walked by dword as the original does (a dword past it refused).
//! Refuses (before anything) a cue past 0x70, the end of the 113 sound lists (the original does
//! not check).

use crate::errors::{BadInput, Refused};
use crate::session::Dword;
use crate::sound::store::{CueQueue, SoundObject};
use rc2k_core::physics::sound_queue::{CuePlacement, ENTRY_DWORDS, check_cue, place, sounds};

/// Dword `i` of the queue, counted as the original walks it: the four header dwords, then
/// the entries' eight each (the first cue's place at -1 reads the entry's count: a run-on
/// kept). One past the sixteen entries is refused.
fn dword(q: &CueQueue, i: u32) -> Result<&Dword, BadInput> {
    Ok(match i {
        0 => &q.first,
        1 => &q.place,
        2 => &q.due,
        3 => &q.count,
        _ => {
            let (e, f) = (
                ((i - 4) / ENTRY_DWORDS) as usize,
                ((i - 4) % ENTRY_DWORDS) as usize,
            );
            let c = q
                .cues
                .get(e)
                .ok_or(BadInput("a dword past the cue queue"))?;
            match f {
                0 => &c.first,
                1 => &c.count,
                _ => &c.sounds[f - 2],
            }
        }
    })
}

/// `cue` queued in `object`'s queue (at +0x298), first or last. The registers it leaves
/// no caller reads (both callers keep theirs); the sound it stops on the way (a queue
/// begun again) goes as a volume of 0, of which the audio thread reads only the sound.
// Converted: `rc2k_core::physics::sound_queue::{check_cue, place, sounds}`.
pub fn queue_sound_cue(first: bool, cue: u32, object: &SoundObject) -> Result<(), Refused> {
    check_cue(cue)?;
    let q = &object.cue_queue;
    let count = q.count.get();
    // The entry's first dword (the original's +0x10 + n x 0x20, in dwords).
    let mut entry = 4u32;
    match place(count, first) {
        CuePlacement::Full => return Ok(()),
        CuePlacement::At(n) => entry = entry.wrapping_add(n.wrapping_mul(ENTRY_DWORDS)),
        CuePlacement::First { count } => {
            // The playing sound: the first cue's sound at its place (+0x18 + place x 4).
            let playing = dword(q, q.place.get().wrapping_add(6))?.get();
            crate::sound::turn_parts::play(playing, 0, 0, [0; 3]);
            q.place.set(u32::MAX);
            // The entries moved up one, from the last dword down (`std`, `rep movsd`).
            let n = count.wrapping_mul(ENTRY_DWORDS);
            for k in 0..n {
                let from = dword(q, entry + n - 1 - k)?.get();
                dword(q, entry + n - 1 - k + ENTRY_DWORDS)?.set(from);
            }
        }
    }
    q.count.set(q.count.get().wrapping_add(1));
    dword(q, entry)?.set(cue);
    let list = crate::sound::sets_store::CueLists::list(cue)?;
    let failed = std::cell::Cell::new(None);
    let (written, n) = sounds(
        |k| {
            list.get(k as usize).unwrap_or_else(|e| {
                failed.set(Some(e));
                0
            })
        },
        object.sound_offset.get(),
    );
    if let Some(e) = failed.take() {
        return Err(e.into());
    }
    for (k, &v) in written.iter().enumerate() {
        dword(q, entry + 2 + k as u32)?.set(v);
    }
    dword(q, entry + 1)?.set(n);
    Ok(())
}
