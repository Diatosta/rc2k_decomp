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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::physics::sound_cues::{PLAY, Part, both, copy_of, dispatch_cue};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::queue_sound_cue", rva(0x40EE5C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The steps on `r`.
pub fn queue_sound_cue_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if r.ebx > 0x5F {
        return Err(BadInput("a sound cue past 0x5F").into());
    }
    let q = r.edi;
    unsafe {
        r.ecx = at(q + 0xC);
        r.edx = q + 0x10;
        if r.ecx == 0x10 {
            return Ok(());
        }
        if r.ecx != 0 {
            if r.eax != 0 {
                r.ecx = r.ecx.wrapping_mul(8);
                r.edx = r.edx.wrapping_add(r.ecx.wrapping_mul(4));
            } else {
                let cue = r.ebx;
                r.eax = at(q + 4);
                r.eax = at(q.wrapping_add(r.eax.wrapping_mul(4)).wrapping_add(0x18));
                r.ebx = 0;
                part(PLAY, r)?;
                put(q + 4, u32::MAX);
                r.ebx = cue;
                r.ecx = r.ecx.wrapping_mul(8);
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
        r.ecx = 0;
        loop {
            r.eax = at(r.ebx + 4 * r.ecx);
            if (r.eax as i32) >= 0 {
                r.eax = r.eax.wrapping_add(at(r.esi));
            }
            put(r.edx + 4 * r.ecx + 8, r.eax);
            if r.eax == u32::MAX {
                break;
            }
            r.ecx += 1;
            if r.ecx == 6 {
                break;
            }
        }
        put(r.edx + 4, r.ecx);
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_cue(&HOOK, r, call_original, queue_sound_cue_in);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x40EE5C,
        0x7C,
        &[0x55, 0x8B, 0x4F, 0x0C, 0x8D, 0x57, 0x10],
        &[0x21],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    // The object (its base at +0), then its queue at +0x298 (0x220 bytes).
    let mut cells = vec![0u32; (0x298 + 0x220) / 4];
    let o = cells.as_mut_ptr() as u32;
    let q = o + 0x298;
    let all = [(o, 4 * cells.len())];
    for count in [0u32, 3, 15, 16] {
        for first in [0u32, 1] {
            for cue in [0x50u32, 0x52] {
                let fill = || unsafe {
                    for k in 0..(cells.len() as u32) {
                        put(o + 4 * k, 0x5A5A_0000 | k);
                    }
                    put(o, 0x1000);
                    put(q + 4, count % 4);
                    put(q + 0xC, count);
                };
                both(
                    &mut t,
                    &format!("count {count}, first {first}, cue {cue:#x}"),
                    &copy,
                    Regs {
                        eax: first,
                        ebx: cue,
                        esi: o,
                        edi: q,
                        ..Regs::SENTINEL
                    },
                    &fill,
                    &all,
                    queue_sound_cue_in,
                );
            }
        }
    }
    drop((copy, cells));
    t.finish()
}
