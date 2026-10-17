//! `marks::clamp_pair` (0x4BEF22): the pair in st(0), st(1) scaled to whole numbers, held
//! to 0 .. 0x7FFF and 0 .. 0xFFFF, and scaled back.
//!
//! In:  st(0), st(1) the pair
//! Out: ebx, eax the whole numbers; st(0), st(1) them scaled back (unrounded)
//!
//! See `marks/turns.rs`. Thunk by `pair_hook!`. Callers: `sub_4BEB25` (twice). Self-test:
//! the original and the Rust version on pairs by lot (negative, large, NaN among them).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, rva};
use crate::marks::turns::*;
use crate::race::stage_prefs::left_of;
use crate::scene::vector3::{Pair80, pair_hook};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PAIR: Pair80 = Pair80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::clamp_pair", rva(0x4BEF22), thunk, &ORIGINAL).self_test(self_test);

pair_hook!(
    thunk,
    call_original,
    original_with_pair,
    dispatch = dispatch,
    original = ORIGINAL,
    pair = PAIR
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (g, outputs) = unsafe { clamp_pair(r, PAIR.inputs()) };
    PAIR.set_outputs(outputs);
    Ok(left_of(&g))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, original_with_pair, &[PAIR.output_region()], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    use crate::scene::matrix3::extended;
    let special = [
        [extended(0.5f32.to_bits()), extended(200.0f32.to_bits())],
        [extended((-3.0f32).to_bits()), extended(1.0e9f32.to_bits())],
        [extended(f32::NAN.to_bits()), extended(127.99f32.to_bits())],
    ];
    let pairs = (1..=10u32).map(test_pair).chain(special);
    for (n, pair) in pairs.enumerate() {
        check_state(
            &mut t,
            &format!("pair {n}"),
            original_with_pair,
            Regs::SENTINEL,
            &|| {
                PAIR.set_inputs(pair);
                PAIR.set_outputs([[0xAA; 10]; 2]);
            },
            &[PAIR.output_region()],
            &rust,
        );
    }
    t.finish()
}
