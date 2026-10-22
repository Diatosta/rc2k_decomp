//! `physics::lever_pushes` (0x43DDDF): `dword_6140E4` and `dword_6140E8` 0; then, only
//! with `dword_621884` 1 and `dword_71A22C` 0: the lever `flt_614B44`, `B48` 0 and
//! `flt_614C44` 0; a share A / (B + A) of A = |+0x210| + |+0x21C| and B = |+0x214| +
//! |+0x218| (`g_car`, `dword_71A1E8` .. `dword_71A1F4`; a reading: the wheels' loads)
//! into `flt_614C44`, held to `flt_5C5D90` (0) ..= `flt_5C5D38` (1). If B + A is 0 (or no
//! number) the share is A itself and it ends there. Else `flt_614B4C` = `flt_71ACD4` and
//! `flt_614C48` the share, `physics::lever_push`; `flt_614B4C` = `flt_71ACC8` and
//! `flt_614C48` 1 less it, `physics::lever_push`. It ends through the `retn` at 0x43DFB0.
//!
//! In:  nothing
//! Out: as the second `lever_push` leaves them when it ran, else all kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B831, 0x43B8AA), every physics step.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): both switches, loads by lot and all 0, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{TEST_CONTROL_WORDS, lever_share, one_less, with_control_word};
use crate::physics::lever_push::{inputs, lever_push, regions as push_regions, set_inputs};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::cmp::Ordering;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::lever_pushes", rva(0x43DDDF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn a(x: usize) -> u32 {
    game_addr(rva(x)) as u32
}

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

const SHARE: usize = 0x614C44; // flt_614C44
const ZERO: usize = 0x5C5D90; // flt_5C5D90
const ONE: usize = 0x5C5D38; // flt_5C5D38

/// `fld limit; fld share; fcompp` and its C0: share below limit, or unordered.
fn below(share: u32, limit: u32) -> bool {
    !matches!(
        f32::from_bits(share).partial_cmp(&f32::from_bits(limit)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The Rust version.
pub unsafe fn lever_pushes() -> Left {
    p(0x6140E4, 0);
    p(0x6140E8, 0);
    if g(0x621884) != 1 || g(0x71A22C) != 0 {
        return Left::nothing();
    }
    p(0x614B44, 0);
    p(0x614B48, 0);
    p(SHARE, 0);
    let (share, zero) = lever_share(g(0x71A1E8), g(0x71A1F4), g(0x71A1EC), g(0x71A1F0), g(ZERO));
    p(SHARE, share);
    if zero {
        return Left::nothing();
    }
    if below(g(SHARE), g(ZERO)) {
        p(SHARE, x87::loaded(g(ZERO)));
    }
    if !below(g(SHARE), g(ONE)) {
        p(SHARE, x87::loaded(g(ONE)));
    }
    p(0x614B4C, x87::loaded(g(0x71ACD4)));
    p(0x614C48, x87::loaded(g(SHARE)));
    unsafe { lever_push() };
    p(0x614B4C, x87::loaded(g(0x71ACC8)));
    p(0x614C48, one_less(g(0x614C48)));
    unsafe { lever_push() }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { lever_pushes() })
}

fn regions() -> Vec<Region> {
    let mut r = push_regions();
    r.push((a(0x614B44), 12));
    r.push((a(SHARE), 8));
    r
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
    let mut all = inputs();
    all.extend(regions());
    all.extend([
        (a(0x621884), 4),
        (a(0x71A22C), 4),
        (a(0x71A1E8), 16),
        (a(0x71ACC8), 4),
        (a(0x71ACD4), 4),
    ]);
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for (switch, still) in [(1u32, 0u32), (0, 0), (1, 1), (2, 0)] {
                for seed in 1..=6u32 {
                    let set = || {
                        set_inputs(seed);
                        p(0x621884, switch);
                        p(0x71A22C, still);
                        let mut f = [0u32; 6];
                        test_floats(seed * 7 + switch, &mut f);
                        for (k, &x) in f.iter().take(4).enumerate() {
                            // All four 0 in one case; minus ones in another.
                            let v = match seed {
                                1 => 0,
                                2 => x | 0x8000_0000,
                                _ => x,
                            };
                            p(0x71A1E8 + 4 * k, v);
                        }
                        p(0x71ACC8, f[4]);
                        p(0x71ACD4, f[5]);
                    };
                    check_state(
                        &mut t,
                        &format!("control word {word:#x}, switches {switch}, {still}, seed {seed}"),
                        call_original,
                        Regs::SENTINEL,
                        &set,
                        &all,
                        &rust,
                    );
                }
            }
        });
    }
    restore(&all, &saved);
    t.finish()
}
