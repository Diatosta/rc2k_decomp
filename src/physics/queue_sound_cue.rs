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
//! Caller: `physics::cue_car_sound`.
//! Refuses (before anything) a cue past 0x5F (the list's size not worked out).
//! Shadow mode runs the original alone (`physics/sound_cues.rs`).
//! Self-test: a copy whose `play_sound` goes to a stand-in, against the Rust version on a
//! test queue: counts 0, 3, 15, 16, both ends, two cues.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::physics::sound_cues::{PLAY, Part, cue_entry};
use rc2k_core::physics::sound_queue::{ENTRY_DWORDS, Place, check_cue, place, sounds};

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::sound_queue::{check_cue, place, sounds}`.
pub fn queue_sound_cue_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    check_cue(r.ebx)?;
    let q = r.edi;
    unsafe {
        let count = at(q + 0xC);
        let placed = place(count, r.eax == 0);
        r.ecx = count;
        r.edx = q + 0x10;
        match placed {
            Place::Full => return Ok(()),
            Place::At(n) => {
                r.ecx = n.wrapping_mul(ENTRY_DWORDS);
                r.edx = r.edx.wrapping_add(r.ecx.wrapping_mul(4));
            }
            Place::First { count } => {
                let cue = r.ebx;
                r.eax = at(q + 4);
                r.eax = at(q.wrapping_add(r.eax.wrapping_mul(4)).wrapping_add(0x18));
                r.ebx = 0;
                part(PLAY, r)?;
                put(q + 4, u32::MAX);
                r.ebx = cue;
                r.ecx = count.wrapping_mul(ENTRY_DWORDS);
                let src = r.edx + r.ecx * 4 - 4;
                let dst = r.edx + r.ecx * 4 + 0x1C;
                for k in 0..r.ecx {
                    put(dst - 4 * k, at(src - 4 * k));
                }
                r.ecx = 0;
            }
        }
        put(q + 0xC, at(q + 0xC).wrapping_add(1));
        put(r.edx, r.ebx);
        r.ebx = game_read(rva(0x512508 + 4 * r.ebx as usize));
        let list = r.ebx;
        let (written, n) = sounds(|k| at(list + 4 * k), at(r.esi));
        for (k, &v) in written.iter().enumerate() {
            put(r.edx + 4 * k as u32 + 8, v);
        }
        r.eax = *written.last().unwrap_or(&r.eax);
        r.ecx = n;
        put(r.edx + 4, r.ecx);
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    cue_entry(r, queue_sound_cue_in)
}
