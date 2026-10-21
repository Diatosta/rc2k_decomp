//! `sound::free_spot_voices` (0x40E6F4): the voices not heard this turn freed.
//!
//! In:  esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! For each voice not heard (+4 not 1): a playing one (+0x28 1) whose id is not free has
//! its sound stopped (`sound::stop_sound`, the request's sound = +0x14) and is freed; a
//! playing free one is left; one not playing is freed. Freed: id 0x1888FFFF, sound -2
//! (+0x28 is left as it is). See `sound/spot_voices.rs`. One caller,
//! `sound::spot_sounds`.
//! Refuses (before anything) a null request.
//! Shadow mode runs the original alone (it stops sounds).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version, on the
//! game's voices (put back).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::service::harness::{capture, restore};
use crate::sound::spot_voices::{FREE, VOICE_COUNT, fill_voices, set, voice, voice_regions};
use crate::sound::turn_parts::{
    Part, REQUEST, STOP, both, common_regions, copy_of, dispatch_turn, run_copy,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::free_spot_voices", rva(0x40E6F4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x3F;
const OUTS: [usize; 1] = [0x25];
const ENTRY: [u8; 7] = [0x60, 0x8D, 0x1D, 0xC0, 0x68, 0x51, 0x00];

// ---- Rust version ----

/// Converted: `rc2k_core::sound::spot_voices::free_unheard`; each stop sent with the
/// registers the original has there (the request's sound written first).
pub fn free_spot_voices_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    use crate::sound::spot_voices::{load_voices, store_voices};
    if r.esi == 0 {
        return Err(BadInput("no request").into());
    }
    let mut voices = load_voices();
    let mut c = *r;
    let done = rc2k_core::sound::spot_voices::free_unheard::<Refused>(&mut voices, |n, sound| {
        (c.ebx, c.ecx, c.eax) = (voice(n as u32), VOICE_COUNT - n as u32, sound);
        unsafe { (c.esi as *mut u32).write_unaligned(sound) };
        part(STOP, &mut c)
    });
    store_voices(&voices);
    done
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, free_spot_voices_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40E6F4, LEN, &ENTRY, &OUTS, &[STOP]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let mut all = voice_regions();
    all.extend(common_regions());
    let saved = capture(&all);
    for seed in 0..4u32 {
        let fill = || {
            fill_voices(seed, &|n, v| {
                let k = n.wrapping_add(seed);
                set(v, 4, [1, 0, 2][k as usize % 3]);
                set(v, 0x28, [1, 0, 1, 1, 3][k as usize % 5]);
                set(v, 0, if k % 7 == 0 { FREE } else { 0x300 + n });
            });
        };
        both(
            &mut t,
            &format!("voices {seed}"),
            &run,
            Regs {
                esi: game_addr(rva(REQUEST)) as u32,
                ..Regs::SENTINEL
            },
            &fill,
            &all,
            free_spot_voices_in,
            false,
        );
    }
    restore(&all, &saved);
    t.finish()
}
