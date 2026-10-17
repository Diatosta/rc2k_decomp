//! `marks::keep_vectors` (0x4C1339): twelve dwords kept (`push`/`pop`): `dword_769252` ..
//! and `dword_76924E` into `dword_76926E` .. and `dword_76926A`; `dword_764FB0` .. and
//! `dword_764FAC` into `dword_7692C2` .. and `dword_7692BE`; `dword_764F94` .. and
//! `dword_764F90` into `dword_7692A6` .. and `dword_7692A2`.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1270`. Self-test: the twelve sources by lot, compared on the twelve
//! results; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::leaf_test::test_leaf;
use crate::screens::harness::Left;
use crate::service::harness::{at, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::keep_vectors", rva(0x4C1339), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// (from, to), in the original's order.
const MOVES: [(usize, usize); 12] = [
    (rva(0x769252), rva(0x76926E)),
    (rva(0x769256), rva(0x769272)),
    (rva(0x76925A), rva(0x769276)),
    (rva(0x76924E), rva(0x76926A)),
    (rva(0x764FB0), rva(0x7692C2)),
    (rva(0x764FB4), rva(0x7692C6)),
    (rva(0x764FB8), rva(0x7692CA)),
    (rva(0x764FAC), rva(0x7692BE)),
    (rva(0x764F94), rva(0x7692A6)),
    (rva(0x764F98), rva(0x7692AA)),
    (rva(0x764F9C), rva(0x7692AE)),
    (rva(0x764F90), rva(0x7692A2)),
];

/// The Rust version (see the header).
pub unsafe fn keep_vectors() {
    for (from, to) in MOVES {
        unsafe { game_write(to, game_read::<u32>(from)) };
    }
}

/// What it writes.
pub fn regions() -> Vec<crate::service::harness::Region> {
    MOVES.map(|m| at(m.1)).to_vec()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { keep_vectors() };
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
