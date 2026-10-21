//! `marks::turn_pair` (0x4BF409): the pair in st(0), st(1) turned by `g_marksLineTurn`,
//! left there.
//!
//! In:  st(0), st(1) the pair
//! Out: st(0), st(1) the turned pair (unrounded), every register kept
//!
//! See `marks/turns.rs`. Thunk by `pair_hook!` (two values popped in, two pushed out).
//! Callers: `sub_4BEFD2` (eight). Self-test: the original and the Rust version on pairs and
//! turns by lot, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `turns::turn_pair_plain`, through
//! `rc2k_core::marks::turns::turn_pair` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, rva};
use crate::marks::turns::*;
use crate::race::stage_prefs::left_of;
use crate::scene::vector3::{Pair80, pair_hook};
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PAIR: Pair80 = Pair80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::turn_pair", rva(0x4BF409), thunk, &ORIGINAL).self_test(self_test);

pair_hook!(
    thunk,
    call_original,
    original_with_pair,
    dispatch = dispatch,
    original = ORIGINAL,
    pair = PAIR
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    PAIR.set_outputs(unsafe { turn_pair_plain(PAIR.inputs()) });
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let floats = Floats {
        memory: &[(PAIR.output_region(), EXTENDED)],
        regs: &[],
    };
    dispatch_state_floats(
        &HOOK,
        r,
        original_with_pair,
        &[PAIR.output_region()],
        &floats,
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let turn = turn_regions();
    let saved = capture(&turn);
    let tolerant_places = [(PAIR.output_region(), EXTENDED)];
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for seed in 1..=12u32 {
            check_state_floats(
                &mut t,
                "marks::turn_pair",
                &format!("control word {word:#x}, seed {seed}"),
                original_with_pair,
                Regs::SENTINEL,
                &|| {
                    let mut m = [0u32; 4];
                    crate::scene::matrix3::test_floats(seed + 40, &mut m);
                    for (r, v) in TURN_RVA.iter().zip(m) {
                        unsafe { game_write(*r, v) };
                    }
                    PAIR.set_inputs(test_pair(seed));
                    PAIR.set_outputs([[0xAA; 10]; 2]);
                },
                &[PAIR.output_region()],
                held,
                &rust,
            );
        }
    });
    restore(&turn, &saved);
    t.finish()
}
