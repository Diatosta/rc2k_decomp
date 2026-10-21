//! `service::repair_gearbox` (0x486D59): the current car's gearbox repaired whole.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `service/repair_parts.rs` (`Part::Gearbox`; converted: `rc2k_core::service::car`). One caller, `service::repair_car`. Refuses a
//! missing player. Self-test: the current car, its dwords given test values, put back
//! afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use crate::service::repair_parts::{Part, check_repair, regions, repair_part, runs_of};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::repair_gearbox", rva(0x486D59), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub unsafe fn repair_gearbox() -> Result<(), Refused> {
    Ok(unsafe { repair_part(Part::Gearbox) }?)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { repair_gearbox() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let runs = runs_of(Part::Gearbox);
    if !check_repair(&mut t, call_original, &rust, &runs) {
        return t.skip("there is no current player object");
    }
    t.finish()
}
