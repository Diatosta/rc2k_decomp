//! `marks::spread_point` (0x4C61AB): the three values in st(1) .. st(3) each grown by
//! itself times st(0) times a random times `dbl_76521C` (`x87::grown`), left as st(0) ..
//! st(2).
//!
//! In:  st(0) the spread, st(1) .. st(3) the point (each rounded to a float)
//! Out: st(0) .. st(2) the point grown (st(1)'s first); eax the random as left; the rest
//!      kept
//!
//! The random (`g_marksSpreadRandom`): times 0x9D plus 0x47, then rotated right 12 twice,
//! each of the three a whole number (signed) of it: the third for st(3), the second for
//! st(2), the first for st(1). Thunk by `quad_triple_hook!`. Callers: four in the marks'
//! particles. Self-test: as `marks::jitter_point`'s.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, rva};
use crate::marks::jitter_point::test_inputs;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::vector3::{Quad80, quad_triple_hook};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static QUAD: Quad80 = Quad80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::spread_point", rva(0x4C61AB), thunk, &ORIGINAL).self_test(self_test);

quad_triple_hook!(
    thunk,
    call_original,
    original_with_quad,
    dispatch = dispatch,
    original = ORIGINAL,
    quad = QUAD
);

const RANDOM_RVA: usize = rva(0x923004); // g_marksSpreadRandom
const SCALE_RVA: usize = rva(0x76521C); // dbl_76521C

/// The Rust version (see the header): the three out and eax.
pub unsafe fn spread_point(inputs: [[u8; 10]; 4]) -> ([[u8; 10]; 3], u32) {
    let [spread, b, c, d] = inputs.map(rounded);
    unsafe {
        let first = game_read::<u32>(RANDOM_RVA)
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        let second = first.rotate_right(12);
        let third = second.rotate_right(12);
        game_write(RANDOM_RVA, third);
        let k: u64 = game_read(SCALE_RVA);
        let d = x87::grown(third as i32, k, d, spread);
        let c = x87::grown(second as i32, k, c, spread);
        let b = x87::grown(first as i32, k, b, spread);
        ([extended(b), extended(c), extended(d)], third)
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (out, eax) = unsafe { spread_point(QUAD.inputs()) };
    QUAD.set_outputs(out);
    Ok(left_of(&Regs { eax, ..*r }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        original_with_quad,
        &[QUAD.output_region(), at(RANDOM_RVA)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [QUAD.output_region(), at(RANDOM_RVA)];
    let saved = capture(&[at(RANDOM_RVA)]);
    for seed in 1..=30u32 {
        let inputs = test_inputs(seed);
        let random = seed.wrapping_mul(0x9E37_79B9);
        check_state(
            &mut t,
            &format!("seed {seed}: inputs {inputs:x?}, random {random:#x}"),
            original_with_quad,
            Regs::SENTINEL,
            &|| {
                unsafe { game_write(RANDOM_RVA, random) };
                QUAD.set_inputs(inputs);
                QUAD.set_outputs([[0xAA; 10]; 3]);
            },
            &regions,
            &rust,
        );
    }
    restore(&[at(RANDOM_RVA)], &saved);
    t.finish()
}
