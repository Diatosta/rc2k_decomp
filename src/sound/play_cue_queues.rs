//! `sound::play_cue_queues` (0x40ED13): the spoken cues of the two players' cars.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Unless no sound object is in use (`g_raceSoundObjectCount` 0): `sound::play_cue_queue`
//! for the object of car `dword_514F70`, then unless `dword_514F78` is 1 (one player) for
//! that of `dword_514F74`. One caller, `sound::race_turn`.
//! Refuses (before anything) a car past 7 or a null object; what the queues refuse puts
//! back what was written (a sound already sent stays sent).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose two calls go to a stand-in, against the Rust version.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::service::harness::{Region, at, capture, restore};
use crate::sound::turn_parts::{COUNT, OBJECTS, Part, both, copy_of, dispatch_turn, run_copy};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_cue_queues", rva(0x40ED13), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x37;
const OUTS: [usize; 2] = [0x16, 0x30];
const ENTRY: [u8; 8] = [0x60, 0x83, 0x3D, 0x7C, 0x4F, 0x51, 0x00, 0x00];
const QUEUE: u32 = 0x40ED4A;
/// dword_514F70, dword_514F74: the two players' cars; dword_514F78: 1 with one player.
const FIRST: usize = 0x514F70;
const SECOND: usize = 0x514F74;
const ONE_PLAYER: usize = 0x514F78;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

// ---- Rust version ----

pub fn play_cue_queues_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if g(COUNT) == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::sound::turn::cue_cars`.
    let cars = rc2k_core::sound::turn::cue_cars(g(FIRST), || g(SECOND), g(ONE_PLAYER));
    let object = |n: u32| g(OBJECTS + 4 * n as usize);
    if cars.iter().any(|&n| n > 7 || object(n) == 0) {
        return Err(BadInput("a player's car past 7 or without a sound object").into());
    }
    let mut c = *r;
    for n in cars {
        c.eax = n;
        c.ebx = object(n);
        part(QUEUE, &mut c)?;
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, play_cue_queues_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40ED13, LEN, &ENTRY, &OUTS, &[QUEUE]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let a = |x: usize| game_addr(rva(x)) as u32;
    let all: [Region; 3] = [(a(OBJECTS), 0x20), at(rva(COUNT)), (a(FIRST), 12)];
    let saved = capture(&all);
    for count in [0u32, 2] {
        for (first, second) in [(0u32, 1u32), (3, 7), (8, 0), (1, 9)] {
            for one in [0u32, 1] {
                let fill = || unsafe {
                    for n in 0..8u32 {
                        game_write(rva(OBJECTS + 4 * n as usize), 0x1000 + n * 0x10);
                    }
                    game_write(rva(COUNT), count);
                    game_write(rva(FIRST), first);
                    game_write(rva(SECOND), second);
                    game_write(rva(ONE_PLAYER), one);
                };
                both(
                    &mut t,
                    &format!("count {count}, cars {first} {second}, one player {one}"),
                    &run,
                    Regs::SENTINEL,
                    &fill,
                    &all,
                    play_cue_queues_in,
                    count != 0 && (first > 7 || (one != 1 && second > 7)),
                );
            }
        }
    }
    restore(&all, &saved);
    t.finish()
}
