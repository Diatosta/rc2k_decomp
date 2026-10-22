//! `drive::store_first_car` (0x460CAA): `g_currentCar` 0 and `drive::store_car` (the work
//! car back into the first block).
//!
//! In:  nothing
//! Out: as `store_car` leaves the registers; no flags read
//!
//! Callers: `drive::finish_run`, `sub_4293CB` (twice).
//! Refuses what `store_car` refuses.
//! Self-test: the original and the Rust version on the work car and the first block
//! (saved and put back).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, p};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::store_first_car", rva(0x460CAA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn store_first_car() -> Result<Left, Refused> {
    p(0x71B3F8, 0);
    unsafe { crate::drive::store_car::store_car() }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { store_first_car() }
}

fn regions() -> Vec<crate::service::harness::Region> {
    vec![(a(0x71B3F8), 4), (a(0x70F3EC), 0xFE0), (a(0x719FD8), 0xFE0)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.push((a(0x71B3B8), 0x20));
    let saved = capture(&all);
    let set = || {
        restore(&all, &saved);
        p(0x71B3B8, a(0x70F3EC));
        p(0x71B3F8, 3);
    };
    check_state(
        &mut t,
        "the first car",
        call_original,
        Regs::SENTINEL,
        &set,
        &regions(),
        &rust,
    );
    restore(&all, &saved);
    t.finish()
}
