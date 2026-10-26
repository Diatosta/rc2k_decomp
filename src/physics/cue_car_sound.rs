//! `physics::cue_car_sound` (0x40ECE7): for eax 0 or 1 (unsigned), the sound object of
//! car `dword_514F70[eax]` (`g_raceSoundObjects`); when its +8 is 1,
//! `physics::queue_sound_cue` with eax 0 (first), edi its +0x298, the cue in ebx. All in
//! `pusha` / `popa`.
//!
//! In:  eax which, ebx the cue
//! Out: every register kept; no flags read
//!
//! Caller: `physics::cue_listed_car_sound`. A short jump in its first five bytes: hooked
//! by a private copy (its call relocated) and a hand-made patch.
//! Refuses (before anything) a car past 7 or a null sound object.
//! Shadow mode runs the original alone (`physics/sound_cues.rs`).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version: which 0, 1,
//! 2, objects ready or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::sound_cues::{Part, QUEUE, both, copy_of, dispatch_cue};
use crate::service::harness::{capture, restore};
use rc2k_core::physics::listed_cars::{CuePlace, QUEUE_OFFSET, cue_object};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("physics::cue_car_sound", rva(0x40ECE7), thunk, &ORIGINAL)
    .private_copy(0x2C)
    .relocate_calls(&[0x25])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

/// The steps on `r` (its registers kept).
// Converted: `rc2k_core::physics::listed_cars::cue_object`.
pub fn cue_car_sound_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let Some(o) = cue_object(
        r.eax,
        |which| g(0x514F70 + 4 * which as usize),
        |car| g(0x512A10 + 4 * car as usize),
        |o| unsafe { ((o + 8) as *const u32).read_unaligned() },
    )?
    else {
        return Ok(());
    };
    let mut c = *r;
    c.eax = CuePlace::First as u32;
    c.esi = o;
    c.edi = o + QUEUE_OFFSET;
    part(QUEUE, &mut c)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_cue(&HOOK, r, call_original, cue_car_sound_in);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x40ECE7,
        0x2C,
        &[0x60, 0x83, 0xF8, 0x01, 0x77, 0x24],
        &[0x25],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut object = vec![0u32; 0x20];
    let o = object.as_mut_ptr() as u32;
    let a = |x: usize| game_addr(rva(x)) as u32;
    let all = [(o, 0x80), (a(0x512A10), 0x20)];
    let saved = capture(&all);
    for which in [0u32, 1, 2] {
        for ready in [1u32, 0] {
            let fill = || unsafe {
                for k in 0..8usize {
                    game_write(rva(0x512A10 + 4 * k), o);
                }
                ((o + 8) as *mut u32).write(ready);
            };
            both(
                &mut t,
                &format!("which {which}, ready {ready}"),
                &copy,
                Regs {
                    eax: which,
                    ebx: 0x50,
                    ..Regs::SENTINEL
                },
                &fill,
                &all,
                cue_car_sound_in,
            );
        }
    }
    restore(&all, &saved);
    drop((copy, object));
    t.finish()
}
