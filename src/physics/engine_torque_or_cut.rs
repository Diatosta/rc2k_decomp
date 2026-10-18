//! `physics::engine_torque_or_cut` (0x43CB7D): at or past the engine's top (or unordered)
//! the cut: +0x1F0 (`unk_71A1C8`) 5 and the torque `flt_614748` 0; below it, while +0x1F0
//! counts down the torque stays 0; else `physics::engine_torque`.
//!
//! In:  nothing
//! Out: as `engine_torque` leaves them when it ran; no flags read
//!
//! Caller: `physics::drive_force`.
//! Self-test: `physics::engine_torque`'s (`test_engine`).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::engine_torque::{below, engine_torque, g, p, regions, test_engine};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
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

/// The Rust version.
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
    Ok(unsafe { engine_torque_or_cut() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_engine(&mut t, call_original, &rust);
    t.finish()
}
