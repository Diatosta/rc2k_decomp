//! `physics::damage_notices` (0x4B0241): the work car's damage warnings. For each damage
//! gauge (floats of the work copy, `unk_71A284` .. `unk_71A33C`) a level (`unk_71A398`,
//! `unk_71A340` ..) moved by thresholds, in the original's order of 82 steps (`STEPS`,
//! read from the disassembly): a step for level `from` puts the threshold (a whole number)
//! into `flt_75B36C` (`fild` / `fstp`, every step) and compares the gauge with it
//! (`fld; fcomp`, below or unordered as C0): rising steps, not below, set the level to
//! `to` and queue a notice (`physics::queue_notice`: esi the notice, eax the gauge, ebx
//! the rank); falling steps (the hysteresis of some gauges), below, set it back.
//!
//! In:  nothing
//! Out: eax, ebx, esi the last notice's, edx as `queue_notice` leaves it (kept when none);
//!      the rest kept; no flags read (the caller goes on)
//!
//! Caller: `sub_43B635` (0x43B95C), every physics step.
//! Refuses what `queue_notice` refuses (everything written put back).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): gauges and levels by lot, notices on and off (`dword_71A1F8` bit 2), the queue
//! empty and part full, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87::from_int;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::flow::{a, c0, g, guard, p};
use crate::physics::queue_notice::{Contacts, queue_notice};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::damage_notices", rva(0x4B0241), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `flt_75B36C`: the threshold being compared.
const LIMIT: usize = 0x75B36C;

/// One step: a level, the level it applies at, the threshold, the gauge, the new level;
/// a rising one also the notice and its rank.
#[derive(Clone, Copy)]
enum Step {
    Up {
        level: usize,
        from: u32,
        limit: i32,
        gauge: usize,
        to: u32,
        notice: u32,
        rank: u32,
    },
    Down {
        level: usize,
        from: u32,
        limit: i32,
        gauge: usize,
        to: u32,
    },
}
use Step::{Down, Up};

/// The original's steps, in order.
#[rustfmt::skip]
const STEPS: [Step; 82] = [
    Up { level: 0x71A398, from: 0, limit: 1, gauge: 0x71A28C, to: 1, notice: 0xCF, rank: 1 },
    Down { level: 0x71A398, from: 1, limit: 1, gauge: 0x71A28C, to: 0 },
    Up { level: 0x71A398, from: 1, limit: 50, gauge: 0x71A28C, to: 2, notice: 0xCF, rank: 2 },
    Down { level: 0x71A398, from: 2, limit: 50, gauge: 0x71A28C, to: 1 },
    Up { level: 0x71A398, from: 2, limit: 75, gauge: 0x71A28C, to: 3, notice: 0xD0, rank: 3 },
    Down { level: 0x71A398, from: 3, limit: 75, gauge: 0x71A28C, to: 2 },
    Up { level: 0x71A39C, from: 0, limit: 75, gauge: 0x71A284, to: 1, notice: 0xD1, rank: 1 },
    Down { level: 0x71A39C, from: 1, limit: 75, gauge: 0x71A284, to: 0 },
    Up { level: 0x71A3A0, from: 0, limit: 75, gauge: 0x71A288, to: 1, notice: 0xD1, rank: 1 },
    Down { level: 0x71A3A0, from: 1, limit: 75, gauge: 0x71A288, to: 0 },
    Up { level: 0x71A340, from: 0, limit: 50, gauge: 0x71A290, to: 1, notice: 0xD2, rank: 1 },
    Up { level: 0x71A340, from: 1, limit: 75, gauge: 0x71A290, to: 2, notice: 0xD2, rank: 2 },
    Up { level: 0x71A340, from: 2, limit: 90, gauge: 0x71A290, to: 3, notice: 0xD3, rank: 3 },
    Up { level: 0x71A340, from: 3, limit: 100, gauge: 0x71A290, to: 4, notice: 0xD4, rank: 4 },
    Up { level: 0x71A344, from: 0, limit: 50, gauge: 0x71A294, to: 1, notice: 0xD5, rank: 1 },
    Up { level: 0x71A344, from: 1, limit: 75, gauge: 0x71A294, to: 2, notice: 0xD5, rank: 2 },
    Up { level: 0x71A344, from: 2, limit: 90, gauge: 0x71A294, to: 3, notice: 0xD6, rank: 3 },
    Up { level: 0x71A344, from: 3, limit: 100, gauge: 0x71A294, to: 4, notice: 0xD7, rank: 4 },
    Up { level: 0x71A348, from: 0, limit: 50, gauge: 0x71A298, to: 1, notice: 0xD8, rank: 1 },
    Up { level: 0x71A348, from: 1, limit: 75, gauge: 0x71A298, to: 2, notice: 0xD8, rank: 2 },
    Up { level: 0x71A348, from: 2, limit: 90, gauge: 0x71A298, to: 3, notice: 0xD9, rank: 3 },
    Up { level: 0x71A348, from: 3, limit: 100, gauge: 0x71A298, to: 4, notice: 0xDA, rank: 4 },
    Up { level: 0x71A34C, from: 0, limit: 50, gauge: 0x71A29C, to: 1, notice: 0xDB, rank: 1 },
    Up { level: 0x71A34C, from: 1, limit: 75, gauge: 0x71A29C, to: 2, notice: 0xDB, rank: 2 },
    Up { level: 0x71A34C, from: 2, limit: 90, gauge: 0x71A29C, to: 3, notice: 0xDC, rank: 3 },
    Up { level: 0x71A34C, from: 3, limit: 100, gauge: 0x71A29C, to: 4, notice: 0xDD, rank: 4 },
    Up { level: 0x71A350, from: 0, limit: 50, gauge: 0x71A2A0, to: 1, notice: 0xDE, rank: 1 },
    Up { level: 0x71A350, from: 1, limit: 75, gauge: 0x71A2A0, to: 2, notice: 0xDE, rank: 2 },
    Up { level: 0x71A350, from: 2, limit: 90, gauge: 0x71A2A0, to: 3, notice: 0xDF, rank: 3 },
    Up { level: 0x71A350, from: 3, limit: 100, gauge: 0x71A2A0, to: 4, notice: 0xE0, rank: 4 },
    Up { level: 0x71A354, from: 0, limit: 50, gauge: 0x71A2A4, to: 1, notice: 0xE1, rank: 1 },
    Up { level: 0x71A354, from: 1, limit: 75, gauge: 0x71A2A4, to: 2, notice: 0xE1, rank: 2 },
    Up { level: 0x71A354, from: 2, limit: 90, gauge: 0x71A2A4, to: 3, notice: 0xE2, rank: 3 },
    Up { level: 0x71A354, from: 3, limit: 100, gauge: 0x71A2A4, to: 4, notice: 0xE3, rank: 4 },
    Up { level: 0x71A358, from: 0, limit: 50, gauge: 0x71A2CC, to: 1, notice: 0xE4, rank: 1 },
    Up { level: 0x71A358, from: 1, limit: 75, gauge: 0x71A2CC, to: 2, notice: 0xE4, rank: 2 },
    Up { level: 0x71A358, from: 2, limit: 90, gauge: 0x71A2CC, to: 3, notice: 0xE5, rank: 3 },
    Up { level: 0x71A358, from: 3, limit: 100, gauge: 0x71A2CC, to: 4, notice: 0xE6, rank: 4 },
    Up { level: 0x71A35C, from: 0, limit: 50, gauge: 0x71A2E4, to: 1, notice: 0xE7, rank: 1 },
    Up { level: 0x71A35C, from: 1, limit: 75, gauge: 0x71A2E4, to: 2, notice: 0xE7, rank: 2 },
    Up { level: 0x71A35C, from: 2, limit: 90, gauge: 0x71A2E4, to: 3, notice: 0xE8, rank: 3 },
    Up { level: 0x71A35C, from: 3, limit: 100, gauge: 0x71A2E4, to: 4, notice: 0xE9, rank: 4 },
    Up { level: 0x71A360, from: 0, limit: 50, gauge: 0x71A2E8, to: 1, notice: 0xEA, rank: 1 },
    Up { level: 0x71A360, from: 1, limit: 75, gauge: 0x71A2E8, to: 2, notice: 0xEA, rank: 2 },
    Up { level: 0x71A360, from: 2, limit: 90, gauge: 0x71A2E8, to: 3, notice: 0xEB, rank: 3 },
    Up { level: 0x71A360, from: 3, limit: 100, gauge: 0x71A2E8, to: 4, notice: 0xEC, rank: 4 },
    Up { level: 0x71A364, from: 0, limit: 50, gauge: 0x71A308, to: 1, notice: 0xED, rank: 1 },
    Up { level: 0x71A364, from: 1, limit: 75, gauge: 0x71A308, to: 2, notice: 0xED, rank: 2 },
    Up { level: 0x71A364, from: 2, limit: 90, gauge: 0x71A308, to: 3, notice: 0xEE, rank: 3 },
    Up { level: 0x71A364, from: 3, limit: 100, gauge: 0x71A308, to: 4, notice: 0xEF, rank: 4 },
    Up { level: 0x71A368, from: 0, limit: 50, gauge: 0x71A328, to: 1, notice: 0xF0, rank: 1 },
    Up { level: 0x71A368, from: 1, limit: 75, gauge: 0x71A328, to: 2, notice: 0xF0, rank: 2 },
    Up { level: 0x71A368, from: 2, limit: 90, gauge: 0x71A328, to: 3, notice: 0xF1, rank: 3 },
    Up { level: 0x71A368, from: 3, limit: 100, gauge: 0x71A328, to: 4, notice: 0xF2, rank: 4 },
    Up { level: 0x71A374, from: 0, limit: 50, gauge: 0x71A31C, to: 1, notice: 0xF3, rank: 1 },
    Up { level: 0x71A374, from: 1, limit: 75, gauge: 0x71A31C, to: 2, notice: 0xF3, rank: 2 },
    Up { level: 0x71A374, from: 2, limit: 90, gauge: 0x71A31C, to: 3, notice: 0xF4, rank: 3 },
    Up { level: 0x71A374, from: 3, limit: 100, gauge: 0x71A31C, to: 4, notice: 0xF5, rank: 4 },
    Up { level: 0x71A36C, from: 0, limit: 50, gauge: 0x71A304, to: 1, notice: 0xF6, rank: 1 },
    Up { level: 0x71A36C, from: 1, limit: 75, gauge: 0x71A304, to: 2, notice: 0xF6, rank: 2 },
    Up { level: 0x71A36C, from: 2, limit: 90, gauge: 0x71A304, to: 3, notice: 0xF7, rank: 3 },
    Up { level: 0x71A3A4, from: 0, limit: 100, gauge: 0x71A2EC, to: 1, notice: 0xF8, rank: 1 },
    Up { level: 0x71A3A8, from: 0, limit: 100, gauge: 0x71A2F0, to: 1, notice: 0xF9, rank: 1 },
    Up { level: 0x71A3AC, from: 0, limit: 100, gauge: 0x71A2F4, to: 1, notice: 0xFA, rank: 1 },
    Up { level: 0x71A3B0, from: 0, limit: 100, gauge: 0x71A2F8, to: 1, notice: 0xFB, rank: 1 },
    Up { level: 0x71A3B4, from: 0, limit: 100, gauge: 0x71A2FC, to: 1, notice: 0xFC, rank: 1 },
    Up { level: 0x71A3B8, from: 0, limit: 100, gauge: 0x71A300, to: 1, notice: 0xFD, rank: 1 },
    Up { level: 0x71A3BC, from: 0, limit: 101, gauge: 0x71A2D0, to: 1, notice: 0xFE, rank: 1 },
    Up { level: 0x71A3C0, from: 0, limit: 101, gauge: 0x71A2D4, to: 1, notice: 0xFF, rank: 1 },
    Up { level: 0x71A3C4, from: 0, limit: 101, gauge: 0x71A2D8, to: 1, notice: 0x100, rank: 1 },
    Up { level: 0x71A3C8, from: 0, limit: 101, gauge: 0x71A2DC, to: 1, notice: 0x101, rank: 1 },
    Up { level: 0x71A370, from: 0, limit: 50, gauge: 0x71A33C, to: 1, notice: 0x102, rank: 1 },
    Up { level: 0x71A370, from: 1, limit: 75, gauge: 0x71A33C, to: 2, notice: 0x102, rank: 2 },
    Up { level: 0x71A370, from: 2, limit: 90, gauge: 0x71A33C, to: 3, notice: 0x103, rank: 3 },
    Up { level: 0x71A378, from: 0, limit: 100, gauge: 0x71A2A8, to: 1, notice: 0x104, rank: 1 },
    Up { level: 0x71A37C, from: 0, limit: 100, gauge: 0x71A2B0, to: 1, notice: 0x105, rank: 1 },
    Up { level: 0x71A380, from: 0, limit: 100, gauge: 0x71A2B4, to: 1, notice: 0x106, rank: 1 },
    Up { level: 0x71A384, from: 0, limit: 100, gauge: 0x71A2B8, to: 1, notice: 0x107, rank: 1 },
    Up { level: 0x71A388, from: 0, limit: 100, gauge: 0x71A2BC, to: 1, notice: 0x108, rank: 1 },
    Up { level: 0x71A38C, from: 0, limit: 100, gauge: 0x71A2C0, to: 1, notice: 0x109, rank: 1 },
    Up { level: 0x71A390, from: 0, limit: 100, gauge: 0x71A2C4, to: 1, notice: 0x10A, rank: 1 },
    Up { level: 0x71A394, from: 0, limit: 100, gauge: 0x71A2C8, to: 1, notice: 0x10B, rank: 1 },
];

fn steps(r: &mut Regs) -> Result<(), Refused> {
    for step in STEPS {
        let (level, from, limit, gauge) = match step {
            Up {
                level,
                from,
                limit,
                gauge,
                ..
            }
            | Down {
                level,
                from,
                limit,
                gauge,
                ..
            } => (level, from, limit, gauge),
        };
        if g(level) != from {
            continue;
        }
        p(LIMIT, from_int(limit));
        let below = c0(g(gauge), g(LIMIT));
        match step {
            Up {
                to, notice, rank, ..
            } if !below => {
                p(level, to);
                r.esi = notice;
                r.eax = g(gauge);
                r.ebx = rank;
                unsafe { queue_notice(r) }?.apply(r);
            }
            Down { to, .. } if below => p(level, to),
            _ => {}
        }
    }
    Ok(())
}

/// What it writes: the levels, the threshold, the queue.
fn regions() -> Vec<Region> {
    let mut all = vec![(a(0x71A340), 0x8C), (a(LIMIT), 4)];
    all.extend(Contacts::game().regions());
    all
}

/// The Rust version.
pub unsafe fn damage_notices(entry: &Regs) -> Result<Left, Refused> {
    guard(&regions(), || {
        let mut r = *entry;
        steps(&mut r)?;
        Ok(left_of(&r))
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { damage_notices(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.extend([(a(0x71A284), 0xBC), (a(0x71A1F8), 4)]);
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..16u32 {
                let set = || {
                    let mut f = [0u32; 0x60];
                    test_floats(seed * 13 + 1, &mut f);
                    // The gauges 0 .. 120 (and some unordered); the levels 0 .. 4.
                    for (k, x) in f.iter().enumerate().take(0x2F) {
                        let v = (f32::from_bits(*x).abs() * 15.0) % 120.0;
                        p(
                            0x71A284 + 4 * k,
                            if x % 31 == 0 {
                                0x7FC0_0000
                            } else {
                                v.to_bits()
                            },
                        );
                    }
                    // All 0x23 levels (to 0x71A3C8): both sides start from the same.
                    for k in 0..0x23 {
                        p(0x71A340 + 4 * k, f[0x2F + k] % 5);
                    }
                    p(0x71A1F8, if seed % 4 == 3 { 0 } else { 4 });
                    p(0x71A450, [0u32, 3, 0x1F, 0x20][(seed % 4) as usize]);
                };
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs::SENTINEL,
                    &set,
                    &all,
                    &rust,
                );
            }
        });
    }
    restore(&all, &saved);
    t.finish()
}
