//! `sound::match_spot_voices` (0x40E642): each shared spot given its voice.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! For each of the `g_sharedSoundSpotCount` spots whose kind is 1 .. 0x15: the voice with
//! its id (the first, searching all 0x48), or else the first free one, made its: id,
//! new (+0x20) 1, the kind's sound, kind, bit, repeats, playing and restart 0. No voice
//! free: the spot goes unheard. Then the voice's place = the spot's and heard (+4) 1.
//! See `sound/spot_voices.rs`. One caller, `sound::spot_sounds`.
//! Refuses (before anything) more than 36 spots (the table's size).
//! Shadow mode: both, the voices compared.
//! Self-test: the original against the Rust version on spots and voices of the test's
//! (the game's tables, put back).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::service::harness::{Region, at, capture, restore};
use crate::sound::spot_voices::{
    FREE, MOST_KIND, MOST_SPOTS, SOUND_OF_KIND, SPOT_COUNT, SPOT_LEN, SPOTS, VOICE_COUNT,
    fill_voices, g, p, set, voice, voice_regions, vw,
};
use crate::sound::turn_parts::{Part, both, dispatch_turn};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::match_spot_voices", rva(0x40E642), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub fn match_spot_voices_in(_: &mut Regs, _: Part) -> Result<(), Refused> {
    let count = g(SPOT_COUNT);
    if count > MOST_SPOTS {
        return Err(BadInput("more than 36 shared spots").into());
    }
    let spots = game_addr(rva(SPOTS)) as u32;
    for s in 0..count {
        let spot = spots + s * SPOT_LEN;
        let (id, kind) = (vw(spot, 0), vw(spot, 0x10));
        if !(1..=MOST_KIND).contains(&kind) {
            continue;
        }
        let found = (0..VOICE_COUNT).map(voice).find(|&v| vw(v, 0) == id);
        let v = match found {
            Some(v) => v,
            None => match (0..VOICE_COUNT).map(voice).find(|&v| vw(v, 0) == FREE) {
                Some(v) => {
                    set(v, 0, id);
                    set(v, 0x20, 1);
                    set(v, 0x14, g(SOUND_OF_KIND + 4 * kind as usize));
                    set(v, 0x1C, 1 << kind);
                    set(v, 0x18, kind);
                    for at in [0x2C, 0x28, 0x24] {
                        set(v, at, 0);
                    }
                    v
                }
                None => continue,
            },
        };
        for k in 0..3 {
            set(v, 8 + 4 * k, vw(spot, 4 + 4 * k));
        }
        set(v, 4, 1);
    }
    Ok(())
}

// ---- hook ----

fn regions(_: &Regs) -> Vec<Region> {
    voice_regions()
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(
        &HOOK,
        r,
        call_original,
        match_spot_voices_in,
        Some(&regions),
    );
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut all = voice_regions();
    all.extend([
        (a(SPOTS), (MOST_SPOTS * SPOT_LEN) as usize),
        at(rva(SPOT_COUNT)),
    ]);
    let saved = capture(&all);
    for count in [0u32, 1, 5, 36, 37] {
        for free in [0u32, 1, 3, 0x48] {
            let fill = || {
                fill_voices(count ^ free, &|n, v| {
                    set(v, 0, if n % 0x18 < free { FREE } else { 0x100 + n });
                });
                for s in 0..MOST_SPOTS {
                    let spot = a(SPOTS) + s * SPOT_LEN;
                    set(spot, 0, [0x105, 0x200 + s, FREE, 0x110][s as usize % 4]);
                    for k in 1..4 {
                        set(spot, 4 * k, 0x3F00_0000 + s * 0x1111 + k);
                    }
                    set(spot, 0x10, [0, 1, 0x15, 0x16, 7][s as usize % 5]);
                }
                p(SPOT_COUNT, count);
            };
            both(
                &mut t,
                &format!("spots {count}, free {free}"),
                &|r| unsafe { call_original(r) },
                Regs::SENTINEL,
                &fill,
                &all,
                match_spot_voices_in,
                count > MOST_SPOTS,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
