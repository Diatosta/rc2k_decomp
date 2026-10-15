//! `service::repair_car` (0x486F63): the current player's car as new.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `service/repair_parts.rs`: eight damage dwords (+0xD4 .. +0xFC), then
//! `service::repair_body`, `reset_tyre_wear`, `repair_gearbox`, `repair_brakes`,
//! `repair_lights`, `repair_suspension`, `reset_tyre_wear` again, `flags_278` -1 and
//! `field_24c`'s eight dwords 0. 69 callers (the event loops' starts, `sub_472BAA` for each
//! player, the race's code). Refuses a missing player. Self-test: the current car, the
//! dwords it writes given test values, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use crate::service::repair_parts::{car_runs, check_repair, regions, repair_car};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::repair_car", rva(0x486F63), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { repair_car() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !check_repair(&mut t, call_original, &rust, &car_runs()) {
        return t.skip("there is no current player object");
    }
    t.finish()
}
