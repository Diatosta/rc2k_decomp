//! `marks::scale_vectors_by` (0x4D16A3): two vectors scaled: `flt_764FA0` .. =
//! `dword_764F94` .. by `dword_764F90`, `flt_764FBC` .. = `dword_764FB0` .. by
//! `dword_764FAC` (`fld; fmul`, three on the stack, stored last first).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4D14B4`. Self-test: the eight inputs by lot, the original and the Rust
//! version compared on the six results; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::leaf_test::test_leaf;
use crate::math::x87;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, dispatch_state};
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
    unsafe { scale_vectors_by() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let (inputs, written) = (inputs_of(&PAIRS), results_of(&PAIRS));
    test_leaf(
        &mut t,
        &inputs,
        &written,
        Regs::SENTINEL,
        8,
        call_original,
        &rust,
    );
    t.finish()
}
