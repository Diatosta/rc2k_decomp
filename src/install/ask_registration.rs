//! `install::ask_registration` (0x413650): `g_runRegistration` set, so the registration
//! program runs after the game.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! A list action (`off_51DD60`). Caller `install::run_page` through a page's item. Shadow
//! mode compares. Self-test: against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{RUN_REGISTRATION, p, region};
use crate::race::viewer::{Saved, check_original, dispatch_regs};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::ask_registration", rva(0x413650), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn ask_registration(entry: &Regs) -> Regs {
    p(RUN_REGISTRATION, 1);
    *entry
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(ask_registration(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(
        &HOOK,
        r,
        call_original,
        &[region(RUN_REGISTRATION, 4)],
        0,
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [region(RUN_REGISTRATION, 4)];
    let saved = Saved::new(&regions);
    check_original(
        &mut t,
        "set run registration",
        call_original,
        Regs::SENTINEL,
        &regions,
        0,
        &|| p(RUN_REGISTRATION, 0),
        &rust,
    );
    drop(saved);
    t.finish()
}
