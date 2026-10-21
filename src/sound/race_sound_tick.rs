//! `sound::race_sound_tick` (0x421BA9): the race frame's sound tick: every 20 ms of the
//! scene clock, `sound::race_turn_if`.
//!
//! In:  nothing
//! Out: eax the scene clock less the last tick (not due) or the clock (due); ebx as
//!      `race_turn_if` leaves it; everything else preserved
//! `g_sceneClock` less `dword_5CC3B0` under 0x14 (unsigned): nothing. Else
//! `dword_5CC3B0` = the clock, `sound::race_turn_if`, `dword_5CC3B8` counts up.
//! 28 callers, all in the race's frame (`sub_4541BF`, `sub_45429B`, `sub_4547AE`,
//! `sub_4557AB`, `sub_4559A6`, `sub_442D49`): for the Rust ones, the Rust version
//! (`race_sound_tick_in` with `turn_parts::game_part`); `race_turn`'s quit jump then
//! skips their frames (held for the owner: notes/open_items_full.md).
//! Ungated: the quit jump never comes back.
//! Shadow mode runs the original alone (sounds are sent).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::service::harness::{at, capture, restore};
use crate::sound::turn_parts::{Part, both, copy_of, dispatch_turn, run_copy};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::race_sound_tick", rva(0x421BA9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const LEN: usize = 0x26;
const OUTS: [usize; 1] = [0x1A];
const ENTRY: [u8; 5] = [0xA1, 0x30, 0x38, 0x61, 0x00];
/// g_sceneClock, dword_5CC3B0 the last tick, dword_5CC3B8 the ticks counted.
const CLOCK: usize = 0x613830;
const LAST: usize = 0x5CC3B0;
const TICKS: usize = 0x5CC3B8;
pub const TURN_IF: u32 = 0x421BCF;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

// ---- Rust version ----

pub fn race_sound_tick_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    r.eax = g(CLOCK).wrapping_sub(g(LAST));
    // Converted: `rc2k_core::sound::turn::tick_due`.
    if !rc2k_core::sound::turn::tick_due(g(CLOCK), g(LAST)) {
        return Ok(());
    }
    r.eax = g(CLOCK);
    unsafe { game_write(rva(LAST), r.eax) };
    part(TURN_IF, r)?;
    unsafe { game_write(rva(TICKS), g(TICKS).wrapping_add(1)) };
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, race_sound_tick_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x421BA9, LEN, &ENTRY, &OUTS, &[TURN_IF]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let all = [at(rva(CLOCK)), at(rva(LAST)), at(rva(TICKS))];
    let saved = capture(&all);
    for (clock, last) in [
        (100u32, 81u32),
        (100, 80),
        (5, 0xFFFF_FFF0),
        (5, 0xFFFF_FFF2),
        (0, 0),
    ] {
        let fill = || unsafe {
            game_write(rva(CLOCK), clock);
            game_write(rva(LAST), last);
            game_write(rva(TICKS), 9u32);
        };
        both(
            &mut t,
            &format!("clock {clock:#x}, last {last:#x}"),
            &run,
            Regs::SENTINEL,
            &fill,
            &all,
            race_sound_tick_in,
            false,
        );
    }
    restore(&all, &saved);
    t.finish()
}
