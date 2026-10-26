//! `physics::free_rev_in_neutral` (0x43EA54): `physics::free_rev` in neutral.
//!
//! In:  nothing
//! Out: as `free_rev` leaves them in neutral; no flags read
//!
//! Caller: `sub_43B635` (0x43B7B2), every physics step.
//! Self-test: `physics::engine_torque`'s (`test_engine_floats`), the converted version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::physics::engine_torque::{dispatch_engine, g, regions, test_engine_floats};
use crate::physics::free_rev::{FREE_REV_FLOATS, free_rev, free_rev_plain};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::free_rev_in_neutral",
    rva(0x43EA54),
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

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn free_rev_in_neutral() -> Left {
    if g(0x71A1AC) != 0 {
        return Left::nothing();
    }
    unsafe { free_rev() }
}

/// The converted version (`physics::free_rev_plain`): the hook's.
// Converted: `rc2k_core::physics::engine::free_rev` (through `free_rev_plain`).
pub unsafe fn free_rev_in_neutral_plain() -> Left {
    if g(0x71A1AC) != 0 {
        return Left::nothing();
    }
    unsafe { free_rev_plain() }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { free_rev_in_neutral_plain() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = regions();
    regions.push((game_addr(rva(0x71A1B0)) as u32, 4));
    dispatch_engine(&HOOK, r, call_original, &regions, &FREE_REV_FLOATS, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_engine_floats(
        &mut t,
        "physics::free_rev_in_neutral",
        call_original,
        &rust,
        &FREE_REV_FLOATS,
    );
    t.finish()
}
