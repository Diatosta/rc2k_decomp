//! `physics::engine_torque_or_cut` (0x43CB7D): at or past the engine's top the cut:
//! +0x1F0 (`unk_71A1C8`) 5 and the torque `flt_614748` 0; below it (an unordered compare
//! counts as below: C0, `jb`; IDA 2026-10-21), while +0x1F0 counts down the torque stays
//! 0; else `physics::engine_torque`.
//!
//! In:  nothing
//! Out: as `engine_torque` leaves them when it ran; no flags read
//!
//! Caller: `physics::drive_force`.
//! Self-test: `physics::engine_torque`'s (`test_engine_floats`), the converted version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::engine_torque::{
    TORQUE_FLOATS, below, dispatch_engine, engine_torque, engine_torque_plain, g, p, regions,
    test_engine_floats,
};
use crate::screens::harness::Left;
use rc2k_core::physics::engine::torque_cut;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::engine_torque_or_cut",
    rva(0x43CB7D),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The converted version (`rc2k_core::physics::engine`, plain floats): the hook's.
// Converted: `rc2k_core::physics::engine::torque_cut`.
pub unsafe fn engine_torque_or_cut_plain() -> Left {
    let f = |x: usize| f32::from_bits(g(x));
    let Some(cut) = torque_cut(f(0x71A1B0), f(0x71A1B4), g(0x71A1C8)) else {
        return unsafe { engine_torque_plain() };
    };
    p(0x71A1C8, cut);
    p(0x614748, 0);
    Left::nothing()
}

/// The Rust version (the exact one, for `physics::drive_force`).
pub unsafe fn engine_torque_or_cut() -> Left {
    if !below(g(0x71A1B0), g(0x71A1B4)) {
        p(0x71A1C8, 5);
    } else if g(0x71A1C8) != 0 {
        p(0x71A1C8, g(0x71A1C8).wrapping_sub(1));
    } else {
        return unsafe { engine_torque() };
    }
    p(0x614748, 0);
    Left::nothing()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { engine_torque_or_cut_plain() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_engine(&HOOK, r, call_original, &regions(), &TORQUE_FLOATS, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_engine_floats(
        &mut t,
        "physics::engine_torque_or_cut",
        call_original,
        &rust,
        &TORQUE_FLOATS,
    );
    t.finish()
}
