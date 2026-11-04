//! `physics::queue_sound_cue` (0x40EE5C): a cue (ebx) into a sound object's queue (edi:
//! the object +0x298; +0xC the count, entries of 0x20 bytes from +0x10). Full (16):
//! nothing. eax 0 puts it first: the playing sound (+0x18 [+4]) stopped by
//! `sound::play_sound` with ebx 0, +4 -1, the entries moved up one (`std`, `rep movsd`);
//! else it goes last. The entry: the cue, then its sounds (`off_512508[cue]`, up to six
//! up to a -1; one not below 0 plus the object's base, esi's +0) at +8 .., their number at
//! +4; the count up by one.
//!
//! In:  eax 0 first / else last, ebx the cue, esi the sound object, edi its queue
//! Out: ebx the cue's list, ecx the sounds' number, edx the entry, eax the last sound;
//!      the rest kept; no flags read
//!
//! Caller: `physics::cue_car_sound`. The object is a reference; the queue is its +0x298
//! and every access is checked against the object. Refuses (before anything) a cue past 0x5F (the list's
//! size not worked out).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::sound::store::SoundObject;
use rc2k_core::physics::listed_cars::QUEUE_OFFSET;
use rc2k_core::physics::sound_queue::{ENTRY_DWORDS, Place, check_cue, place, sounds};

const WHAT: &str = "physics::queue_sound_cue's operand";

fn at(p: GamePtr) -> Result<u32, BadInput> {
    p.read::<u32>(WHAT)
}

fn put(p: GamePtr, v: u32) -> Result<(), BadInput> {
    p.write::<u32>(v, WHAT)
}

/// `cue` queued in `object`'s queue (at +0x298), first or last. The registers it leaves
/// no caller reads (both callers keep theirs); the sound it stops on the way (a queue
/// begun again) goes as a volume of 0, of which the audio thread reads only the sound.
// Converted: `rc2k_core::physics::sound_queue::{check_cue, place, sounds}`.
pub fn queue_sound_cue(first: bool, cue: u32, object: &SoundObject) -> Result<(), Refused> {
    check_cue(cue)?;
    let object = GamePtr::to(object);
    let q = object.offset(QUEUE_OFFSET);
    let count = at(q.offset(0xC))?;
    let mut entry = 0x10u32;
    match place(count, first) {
        Place::Full => return Ok(()),
        Place::At(n) => entry = entry.wrapping_add(n.wrapping_mul(ENTRY_DWORDS).wrapping_mul(4)),
        Place::First { count } => {
            let playing = at(q.offset(at(q.offset(4))?.wrapping_mul(4).wrapping_add(0x18)))?;
            crate::sound::turn_parts::play(playing, 0, 0, [0; 3]);
            put(q.offset(4), u32::MAX)?;
            let n = count.wrapping_mul(ENTRY_DWORDS);
            let src = entry + n * 4 - 4;
            let dst = entry + n * 4 + 0x1C;
            for k in 0..n {
                put(q.offset(dst - 4 * k), at(q.offset(src - 4 * k))?)?;
            }
        }
    }
    put(q.offset(0xC), at(q.offset(0xC))?.wrapping_add(1))?;
    put(q.offset(entry), cue)?;
    let list = crate::sound::sets_store::CueLists::list(cue)?;
    let failed = std::cell::Cell::new(None);
    let (written, n) = sounds(
        |k| {
            list.offset(4 * k).read::<u32>(WHAT).unwrap_or_else(|e| {
                failed.set(Some(e));
                0
            })
        },
        at(object)?,
    );
    if let Some(e) = failed.take() {
        return Err(e.into());
    }
    for (k, &v) in written.iter().enumerate() {
        put(q.offset(entry + 4 * k as u32 + 8), v)?;
    }
    put(q.offset(entry + 4), n)?;
    Ok(())
}
