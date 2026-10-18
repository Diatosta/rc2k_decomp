//! `drive::play_race` (0x444CEE): "CALLING DOPLAYGAME - START" logged between rules, the
//! log off (`g_logEnabled` 0) for `drive::race_loop`, back on, "... - FINIS" between
//! rules.
//!
//! In:  nothing
//! Out: the registers as `race_loop` leaves them; no flags read
//!
//! Callers: `race::run_race`, `sub_443F96`, 0x44402F, `sub_463666`, `sub_4640AA`. It
//! begins with a call: hooked by a private copy of its 0x58 bytes (seven calls relocated)
//! and a hand-made patch; ungated (it holds the race's step loop).
//! Refuses what `race_loop` refuses. Shadow mode runs the original alone.
//! Self-test: skipped (it plays a race).

use crate::drive::race_parts::log_line;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::p;
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("drive::play_race", rva(0x444CEE), thunk, &ORIGINAL)
    .private_copy(0x58)
    .relocate_calls(&[0x0, 0xC, 0x19, 0x29, 0x3F, 0x4C, 0x52])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const EMPTY: usize = 0x4E00B7; // what `log_newline_l1` writes

/// The Rust version.
pub unsafe fn play_race(entry: &Regs) -> Result<Left, Refused> {
    log_line(EMPTY);
    log_line(0x4DC41B);
    log_line(0x4DC436);
    p(0x4E0094, 0);
    let left = unsafe { crate::drive::race_loop::race_loop(entry) }?;
    p(0x4E0094, 1);
    log_line(0x4DC451);
    log_line(0x4DC46C);
    log_line(EMPTY);
    Ok(left)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { play_race(r) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    t.skip("it plays a race")
}
