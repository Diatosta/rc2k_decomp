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
//!
//! Converted 2026-10-21: the hook's Rust version is `spread_point_plain`, through
//! `rc2k_core::marks::particles::spread` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, rva};
use crate::marks::jitter_point::test_inputs;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::vector3::{Quad80, quad_triple_hook};
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{at, capture, restore};
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

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::particles::spread`
pub unsafe fn spread_point_plain(inputs: [[u8; 10]; 4]) -> ([[u8; 10]; 3], u32) {
    use rc2k_core::math::extended::{from_f64, to_f64};
    let [spread, b, c, d] = inputs.map(|v| to_f64(v) as f32);
    unsafe {
        let scale = f64::from_bits(game_read(SCALE_RVA));
        let (point, third) =
            rc2k_core::marks::particles::spread(spread, [b, c, d], game_read(RANDOM_RVA), scale);
        game_write(RANDOM_RVA, third);
        (point.map(|v| from_f64(v as f64)), third)
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (out, eax) = unsafe { spread_point_plain(QUAD.inputs()) };
    QUAD.set_outputs(out);
    Ok(left_of(&Regs { eax, ..*r }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let floats = Floats {
        memory: &[(QUAD.output_region(), EXTENDED)],
        regs: &[],
    };
    dispatch_state_floats(
        &HOOK,
        r,
        original_with_quad,
        &[QUAD.output_region(), at(RANDOM_RVA)],
        &floats,
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [QUAD.output_region(), at(RANDOM_RVA)];
    let saved = capture(&[at(RANDOM_RVA)]);
    let tolerant_places = [(QUAD.output_region(), EXTENDED)];
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for seed in 1..=30u32 {
            let inputs = test_inputs(seed);
            let random = seed.wrapping_mul(0x9E37_79B9);
            check_state_floats(
                &mut t,
                "marks::spread_point",
                &format!(
                    "control word {word:#x}, seed {seed}: inputs {inputs:x?}, random {random:#x}"
                ),
                original_with_quad,
                Regs::SENTINEL,
                &|| {
                    unsafe { game_write(RANDOM_RVA, random) };
                    QUAD.set_inputs(inputs);
                    QUAD.set_outputs([[0xAA; 10]; 3]);
                },
                &regions,
                held,
                &rust,
            );
        }
    });
    restore(&[at(RANDOM_RVA)], &saved);
    t.finish()
}
