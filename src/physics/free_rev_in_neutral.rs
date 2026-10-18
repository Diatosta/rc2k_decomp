//! `physics::free_rev_in_neutral` (0x43EA54): `physics::free_rev` in neutral.
//!
//! In:  nothing
//! Out: as `free_rev` leaves them in neutral; no flags read
//!
//! Caller: `sub_43B635` (0x43B7B2), every physics step.
//! Self-test: `physics::engine_torque`'s (`test_engine`).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::physics::engine_torque::{g, regions, test_engine};
use crate::physics::free_rev::free_rev;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
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

/// The Rust version.
pub unsafe fn free_rev_in_neutral() -> Left {
    if g(0x71A1AC) != 0 {
        return Left::nothing();
    }
    unsafe { free_rev() }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { free_rev_in_neutral() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = regions();
    regions.push((game_addr(rva(0x71A1B0)) as u32, 4));
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_engine(&mut t, call_original, &rust);
    t.finish()
}
