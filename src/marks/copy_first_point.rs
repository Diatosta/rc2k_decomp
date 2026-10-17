//! `marks::copy_first_point` (0x4C1163): `dword_769306` .. `dword_76930E` into
//! `flt_811BDF` .. `flt_811BE7`, and `dword_7692F6` into `dword_811BCF` (`push`/`pop`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1148`. Self-test: the four sources by lot, the original and the Rust
//! version compared on the four results; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::leaf_test::test_leaf;
use crate::screens::harness::Left;
use crate::service::harness::{at, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::copy_first_point", rva(0x4C1163), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// (from, to): the point's three floats, then `dword_7692F6`.
const MOVES: [(usize, usize); 4] = [
    (rva(0x769306), rva(0x811BDF)),
    (rva(0x76930A), rva(0x811BE3)),
    (rva(0x76930E), rva(0x811BE7)),
    (rva(0x7692F6), rva(0x811BCF)),
];

/// The Rust version (see the header).
pub unsafe fn copy_first_point() {
    for (from, to) in MOVES {
        unsafe { game_write(to, game_read::<u32>(from)) };
    }
}

/// What it writes.
pub fn regions() -> Vec<crate::service::harness::Region> {
    MOVES.map(|m| at(m.1)).to_vec()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { copy_first_point() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &MOVES.map(|m| at(m.1)), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_leaf(
        &mut t,
        &MOVES.map(|m| m.0),
        &MOVES.map(|m| m.1),
        Regs::SENTINEL,
        6,
        call_original,
        &rust,
    );
    t.finish()
}
