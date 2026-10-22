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
use crate::physics::flow::{a, g, guard, p};
use crate::physics::queue_notice::{Contacts, queue_notice};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use rc2k_core::physics::damage_notices::{LEVELS, damage_notices as damage_notices_core};
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

const LEVELS_AT: usize = 0x71A340; // unk_71A340: the levels
const GAUGES_AT: usize = 0x71A284; // unk_71A284: the gauges

// Converted: `rc2k_core::physics::damage_notices::damage_notices`.
fn steps(r: &mut Regs) -> Result<(), Refused> {
    let mut levels: [u32; LEVELS] = std::array::from_fn(|k| g(LEVELS_AT + 4 * k));
    let result = damage_notices_core(
        &mut levels,
        |k| f32::from_bits(g(GAUGES_AT + 4 * k)),
        |t| p(LIMIT, from_int(t)),
        |q| {
            r.esi = q.notice;
            r.eax = g(GAUGES_AT + 4 * q.gauge);
            r.ebx = q.rank;
            unsafe { queue_notice(r) }?.apply(r);
            Ok::<(), Refused>(())
        },
    );
    for (k, &v) in levels.iter().enumerate() {
        p(LEVELS_AT + 4 * k, v);
    }
    result
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
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
