//! `drive::race_tick` (0x4455A9, three chunks): one tick of the race. Fatal error
//! 0x80600655 (no message) when the first car block's +0x260 is 0 (its third chunk;
//! the second is the shared fatal reporter). Free roaming (`dword_621884` 0):
//! `drive::simple_car_step`. Else the tick count `dword_6235F8` + 1 and
//! `drive::replay_step` every tick, or with `dword_63DB84` set every second tick, and
//! past 1 every fourth.
//!
//! In:  nothing
//! Out: the registers as the step leaves them; no flags read
//!
//! Caller: `drive::race_step`.
//! Raises 0x80600655 as the original; refuses what the steps refuse.
//! Self-test: skipped (its steps carry their own tests; a race's ticks in play check it).

use crate::errors::{FatalError, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::{g, p};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::race_tick", rva(0x4455A9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const B: usize = 0x70F3EC;

/// The Rust version.
// Converted: `rc2k_core::drive::race_ticks::{tick_runs, NO_CAR_FATAL}`.
pub unsafe fn race_tick(entry: &Regs) -> Result<Left, Refused> {
    if g(B + 0x260) == 0 {
        return Err(FatalError {
            code: rc2k_core::drive::race_ticks::NO_CAR_FATAL,
            message: "",
        }
        .into());
    }
    if g(0x621884) == 0 {
        return unsafe { crate::drive::simple_car_step::simple_car_step(entry) };
    }
    p(0x6235F8, g(0x6235F8).wrapping_add(1));
    if !rc2k_core::drive::race_ticks::tick_runs(g(0x6235F8), g(0x63DB84)) {
        return Ok(Left::nothing());
    }
    let left = unsafe { crate::drive::replay_step::replay_step(entry) }?;
    let mut r = *entry;
    left.apply(&mut r);
    Ok(left_of(&r))
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { race_tick(r) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    t.skip("its steps (simple_car_step, replay_step) carry their own tests; a whole race tick in play checks it")
}
