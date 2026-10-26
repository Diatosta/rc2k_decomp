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
//! Caller: `physics::cue_car_sound`. Refuses (before anything) a cue past 0x5F (the list's
//! size not worked out).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use rc2k_core::physics::sound_queue::{ENTRY_DWORDS, Place, check_cue, place, sounds};

fn at(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::queue_sound_cue's operand")
}

fn put(a: u32, v: u32) {
    crate::pointers::write_or_end::<u32>(a, v, "physics::queue_sound_cue's operand")
}

/// `cue` queued in `object`'s queue (at `queue`), first or last. The registers it leaves
/// no caller reads (both callers keep theirs); the sound it stops on the way (a queue
/// begun again) goes as a volume of 0, of which the audio thread reads only the sound.
// Converted: `rc2k_core::physics::sound_queue::{check_cue, place, sounds}`.
pub fn queue_sound_cue(first: bool, cue: u32, object: u32, queue: u32) -> Result<(), Refused> {
    check_cue(cue)?;
    let q = queue;
    let count = at(q + 0xC);
    let mut entry = q + 0x10;
    match place(count, first) {
        Place::Full => return Ok(()),
        Place::At(n) => entry = entry.wrapping_add(n.wrapping_mul(ENTRY_DWORDS).wrapping_mul(4)),
        Place::First { count } => {
            let playing = at(q.wrapping_add(at(q + 4).wrapping_mul(4)).wrapping_add(0x18));
            crate::sound::turn_parts::play(playing, 0, 0, [0; 3]);
            put(q + 4, u32::MAX);
            let n = count.wrapping_mul(ENTRY_DWORDS);
            let src = entry + n * 4 - 4;
            let dst = entry + n * 4 + 0x1C;
            for k in 0..n {
                put(dst - 4 * k, at(src - 4 * k));
            }
        }
    }
    put(q + 0xC, at(q + 0xC).wrapping_add(1));
    put(entry, cue);
    let list: u32 = crate::sound::sets_store::CUE_LISTS.table[cue as usize].get();
    let (written, n) = sounds(|k| at(list + 4 * k), at(object));
    for (k, &v) in written.iter().enumerate() {
        put(entry + 4 * k as u32 + 8, v);
    }
    put(entry + 4, n);
    Ok(())
}
