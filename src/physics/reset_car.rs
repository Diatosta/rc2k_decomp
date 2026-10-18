//! `physics::reset_car` (0x43EFEF): the work car put back on the line. Its checkpoint
//! (`dword_71A06C`, `g_car` +0x94) into `dword_6365E4`, `dword_6365EC` 0. With
//! `dword_62389C` 2 a free checkpoint is looked for: offsets d = 0, 1, .. 0x3F
//! (`dword_614C4C`), each tried as +d then -d (`dword_614C50` = the checkpoint + it), a
//! try failing when `physics::reset_point_blocked` gives CF for any of the blocks at
//! 0x7103CC, 0x7113AC, 0x71238C (tried in that order, the first CF ends it). The first
//! free one goes into `dword_6365E4`; with none, "cant reset car anywhere - waiting a sec"
//! is logged (`log_write_l1_nl`, inside `pusha` / `popa`), +0x1E8 (`dword_71A1C0`, the
//! stuck count) less 100, and it returns. Else `race::place_car_on_line`,
//! `physics::reset_car_on_line`, and +0xFAC (`dword_71AF84`) 10.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: as the last call leaves them (eax the last try's checkpoint and esi the last
//!      block when none was free); no flags read (every caller returns next)
//!
//! Callers: `physics::reset_when_stuck`, `sub_43A0D3`, `sub_43BDD1`.
//! Refuses what the calls refuse. Shadow mode runs the original alone
//! (`physics/reset_parts.rs`).
//! Self-test: a copy whose calls go to the stand-ins (place 13's carry by a pattern),
//! against the Rust version with the same: modes 2 and others, patterns free at once,
//! after some tries, and never (the log line, a stand-in too); all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::reset_parts::{BLOCKED, LOG, ON_LINE, both, copy_of, dispatch_reset, regions};
use crate::race::car_starts::{Case, Part, prepare};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::reset_car", rva(0x43EFEF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LINE_PLACE: usize = rva(0x6365E4); // dword_6365E4
const OFFSET: usize = rva(0x614C4C); // dword_614C4C
const CANDIDATE: usize = rva(0x614C50); // dword_614C50
const STUCK: usize = rva(0x71A1C0); // g_car + 0x1E8
const BLOCKS: [usize; 3] = [0x7103CC, 0x7113AC, 0x71238C];
const NOWHERE_TEXT: usize = rva(0x4DBB01); // "cant reset car anywhere - waiting a sec"

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}

fn p(a: usize, v: u32) {
    unsafe { game_write(a, v) }
}

/// One try: the candidate set; true when a block says CF.
fn blocked(r: &mut Regs, part: Part) -> Result<bool, Refused> {
    r.eax = g(LINE_PLACE).wrapping_add(g(OFFSET));
    p(CANDIDATE, r.eax);
    for b in BLOCKS {
        r.esi = game_addr(rva(b)) as u32;
        part(BLOCKED, r)?;
        if r.carry() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The Rust version, the calls through `part` (places 13, 14, 6, 16).
pub unsafe fn reset_car(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = g(rva(0x71A06C));
    p(LINE_PLACE, r.eax);
    p(rva(0x6365EC), 0);
    if g(rva(0x62389C)) == 2 {
        p(OFFSET, 0);
        loop {
            if !blocked(&mut r, part)? {
                break;
            }
            p(OFFSET, g(OFFSET).wrapping_neg());
            if !blocked(&mut r, part)? {
                break;
            }
            p(OFFSET, g(OFFSET).wrapping_neg());
            p(OFFSET, g(OFFSET).wrapping_add(1));
            if g(OFFSET) == 0x40 {
                let mut l = r;
                l.eax = game_addr(NOWHERE_TEXT) as u32;
                part(LOG, &mut l)?;
                p(STUCK, g(STUCK).wrapping_sub(100));
                return Ok(r);
            }
        }
        r.eax = g(CANDIDATE);
        p(LINE_PLACE, r.eax);
    }
    part(6, &mut r)?;
    part(ON_LINE, &mut r)?;
    p(rva(0x71AF84), 10);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_reset(&HOOK, r, call_original, reset_car);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x43EFEF,
        0xF0,
        &[0xA1, 0x6C, 0xA0, 0x71, 0x00],
        &[0x41, 0x4E, 0x5B, 0x80, 0x8D, 0x9A, 0xC3, 0xDB, 0xE0],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = regions();
    let saved = capture(&all);
    let patterns: [u64; 6] = [
        0,
        0b1,
        0b1011,
        0x5555_5555_5555_5555,
        0xFFFF_FFFF_FFFF_FFF0,
        u64::MAX,
    ];
    for mode in [2u32, 0, 3] {
        for (n, pattern) in patterns.into_iter().enumerate() {
            let seed = mode * 8 + n as u32;
            let fill = || {
                prepare(
                    &Case {
                        mode,
                        no_place: 0,
                        side: 0,
                        cars_2: 1,
                        current: seed % 4,
                        cars: 4,
                    },
                    seed.wrapping_mul(0x2026_1017),
                );
                p(OFFSET, 0x5A5A_5A5A);
                p(CANDIDATE, 0xA5A5_A5A5);
            };
            both(
                &mut t,
                &format!("mode {mode}, pattern {pattern:#x}"),
                &copy,
                pattern,
                Regs::SENTINEL,
                &fill,
                reset_car,
            );
        }
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
