//! `marks::scale_vectors_by` (0x4D16A3): two vectors scaled: `flt_764FA0` .. =
//! `dword_764F94` .. by `dword_764F90`, `flt_764FBC` .. = `dword_764FB0` .. by
//! `dword_764FAC` (`fld; fmul`, three on the stack, stored last first).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4D14B4`. Self-test: the eight inputs by lot, the original and the Rust
//! version compared on the six results; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `scale_triples_plain`, through
//! `rc2k_core::marks::view::scaled` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::leaf_test::test_leaf_floats;
use crate::math::x87;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{Region, at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::scale_vectors_by", rva(0x4D16A3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// (the factor, the vector, the result).
const PAIRS: [(usize, usize, usize); 2] = [
    (rva(0x764F90), rva(0x764F94), rva(0x764FA0)),
    (rva(0x764FAC), rva(0x764FB0), rva(0x764FBC)),
];

/// The Rust version (see the header).
pub unsafe fn scale_vectors_by() {
    unsafe { scale_triples(&PAIRS) }
}

/// Each (factor, vector, result): the vector's three floats by the factor (`fld; fmul`).
pub unsafe fn scale_triples(pairs: &[(usize, usize, usize)]) {
    unsafe {
        for &(factor, from, to) in pairs {
            let k: u32 = game_read(factor);
            for n in 0..3 {
                game_write(to + 4 * n, x87::mul(game_read(from + 4 * n), k));
            }
        }
    }
}

/// The converted `scale_triples` (plain floats): the two hooks'.
// Converted: `rc2k_core::marks::view::scaled`
pub unsafe fn scale_triples_plain(pairs: &[(usize, usize, usize)]) {
    unsafe {
        for &(factor, from, to) in pairs {
            let k = f32::from_bits(game_read(factor));
            let v: [u32; 3] = game_read(from);
            let made = rc2k_core::marks::view::scaled(v.map(f32::from_bits), k);
            game_write(to, made.map(f32::to_bits));
        }
    }
}

/// The results' dwords.
pub fn results_of(pairs: &[(usize, usize, usize)]) -> Vec<usize> {
    pairs
        .iter()
        .flat_map(|&(_, _, to)| [to, to + 4, to + 8])
        .collect()
}

/// The inputs' dwords.
pub fn inputs_of(pairs: &[(usize, usize, usize)]) -> Vec<usize> {
    pairs
        .iter()
        .flat_map(|&(k, from, _)| [k, from, from + 4, from + 8])
        .collect()
}

pub fn regions() -> Vec<Region> {
    results_of(&PAIRS).into_iter().map(at).collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { scale_triples_plain(&PAIRS) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places(&regions());
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let (inputs, written) = (inputs_of(&PAIRS), results_of(&PAIRS));
    test_leaf_floats(
        &mut t,
        "marks::scale_vectors_by",
        &inputs,
        &written,
        Regs::SENTINEL,
        &[],
        8,
        call_original,
        &rust,
    );
    t.finish()
}
