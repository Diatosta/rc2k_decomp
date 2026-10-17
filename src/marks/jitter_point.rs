//! `marks::jitter_point` (0x4C6106): the three values in st(1) .. st(3) each moved by the
//! remainder of a random by st(0) (`fprem`), left as st(0) .. st(2).
//!
//! In:  st(0) the spread, st(1) .. st(3) the point (each rounded to a float)
//! Out: st(0) .. st(2) the point moved (st(1)'s first); ebx the last random >> 12
//!      (arithmetic); the rest kept
//!
//! Three steps of the marks' random (times 0x9D plus 0x47, shifted right 12): the first
//! moves st(3), the second st(2), the third st(1). Thunk by `quad_triple_hook!`. Nine callers
//! in the marks' particles. Self-test: the original and the Rust version on spreads,
//! points and randoms by lot; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, rva};
use crate::marks::setup::SEED_RVA;
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
    RawHook::new("marks::jitter_point", rva(0x4C6106), thunk, &ORIGINAL).self_test(self_test);

quad_triple_hook!(
    thunk,
    call_original,
    original_with_quad,
    dispatch = dispatch,
    original = ORIGINAL,
    quad = QUAD
);

/// The Rust version (see the header): the four as st(0) .. st(3), the three out the same
/// way; the last random >> 12.
pub unsafe fn jitter_point(inputs: [[u8; 10]; 4]) -> ([[u8; 10]; 3], u32) {
    let [spread, b, c, d] = inputs.map(rounded);
    let mut moved = [d, c, b];
    let mut last = 0;
    unsafe {
        for v in moved.iter_mut() {
            let s = game_read::<u32>(SEED_RVA)
                .wrapping_mul(0x9D)
                .wrapping_add(0x47);
            game_write(SEED_RVA, s);
            last = ((s as i32) >> 12) as u32;
            *v = x87::prem_plus(last as i32, spread, *v);
        }
    }
    let [d, c, b] = moved;
    ([extended(b), extended(c), extended(d)], last)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (out, ebx) = unsafe { jitter_point(QUAD.inputs()) };
    QUAD.set_outputs(out);
    Ok(left_of(&Regs { ebx, ..*r }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        original_with_quad,
        &[QUAD.output_region(), at(SEED_RVA)],
        &rust,
    );
}

/// Four inputs by lot for the tests of this and `marks::spread_point`: the first small
/// and not 0, the others floats or unrounded.
pub fn test_inputs(seed: u32) -> [[u8; 10]; 4] {
    let mut f = [0u32; 8];
    crate::scene::matrix3::test_floats(seed * 13 + 1, &mut f);
    let spread = [0.5f32, 3.0, 0.01, 250.0, -2.0][seed as usize % 5].to_bits();
    [
        if seed.is_multiple_of(3) {
            x87::sub_80(spread, f[7])
        } else {
            extended(spread)
        },
        extended(f[0]),
        x87::sub_80(f[1], f[4]),
        extended(f[2]),
    ]
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [QUAD.output_region(), at(SEED_RVA)];
    let saved = capture(&[at(SEED_RVA)]);
    for seed in 1..=30u32 {
        let inputs = test_inputs(seed);
        let random = seed.wrapping_mul(0x9E37_79B9);
        check_state(
            &mut t,
            &format!("seed {seed}: inputs {inputs:x?}, random {random:#x}"),
            original_with_quad,
            Regs::SENTINEL,
            &|| {
                unsafe { game_write(SEED_RVA, random) };
                QUAD.set_inputs(inputs);
                QUAD.set_outputs([[0xAA; 10]; 3]);
            },
            &regions,
            &rust,
        );
    }
    restore(&[at(SEED_RVA)], &saved);
    t.finish()
}
