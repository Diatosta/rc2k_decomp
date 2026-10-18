//! `drive::finish_race_0` (0x4278C7): `dword_621870` 0 and `drive::finish_race`.
//!
//! In:  nothing
//! Out: as `finish_race` leaves the registers; no flags read
//!
//! Callers: `sub_488155`, `sub_4A14CB`, `sub_4A9439` (the championship's flow). Ungated.
//! Refuses what `finish_race` refuses. Shadow mode runs the original alone.
//! Self-test: skipped (it loads a stage and plays the finish routine).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::p;
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::finish_race_0", rva(0x4278C7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

/// The Rust version.
pub unsafe fn finish_race_0(entry: &Regs) -> Result<Left, Refused> {
    p(0x621870, 0);
    unsafe { crate::drive::finish_race::finish_race(entry) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { finish_race_0(r) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    t.skip("it loads a stage and plays the finish routine")
}
