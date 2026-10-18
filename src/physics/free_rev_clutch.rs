//! `physics::free_rev_clutch` (0x43EA63): in gear with the clutch out (+0x250,
//! `unk_71A228`): `physics::free_rev`, then `physics::wheels_from_revs`.
//!
//! In:  nothing
//! Out: ebx the gear (`wheels_from_revs`), and what `free_rev` leaves; no flags read
//!
//! Caller: `sub_43B635` (0x43B7B7), every physics step.
//! Refuses (before anything) a gear outside -1 ..= 7.
//! Self-test: `physics::engine_torque`'s (`test_engine`), with the clutch word set.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::physics::engine_torque::{g, p, regions, test_engine};
use crate::physics::free_rev::free_rev;
use crate::physics::gearing::{GearPlaces, wheels_from_revs_at};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::free_rev_clutch", rva(0x43EA63), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn free_rev_clutch(entry: &Regs) -> Result<Left, Refused> {
    let gear = g(0x71A1AC);
    if gear == 0 || g(0x71A228) == 0 {
        return Ok(Left::nothing());
    }
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    let mut r = *entry;
    unsafe {
        free_rev().apply(&mut r);
        wheels_from_revs_at(&GearPlaces::game())?.apply(&mut r);
    }
    Ok(crate::race::stage_prefs::left_of(&r))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { free_rev_clutch(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = regions();
    let a = |x: usize| game_addr(rva(x)) as u32;
    regions.extend([(a(0x71A1B0), 4), (a(0x71A190), 8)]);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let a = |x: usize| game_addr(rva(x)) as u32;
    let saved = crate::service::harness::capture(&[(a(0x71A228), 4), (a(0x71A190), 8)]);
    p(0x71A228, 1);
    test_engine(&mut t, call_original, &rust);
    crate::service::harness::restore(&[(a(0x71A228), 4), (a(0x71A190), 8)], &saved);
    t.finish()
}
