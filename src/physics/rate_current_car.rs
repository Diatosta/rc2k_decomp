//! `physics::rate_current_car` (0x4AB5D0): `call physics::rate_damage; retn`.
//!
//! In:  nothing
//! Out: as `rate_damage` (eax the gear bits, ecx 9, esi 0x75B568)
//!
//! Caller: `physics::grade_or_reset` (0x43BA9E, `dword_71A234` not 2).
//! Self-test: a copy of the original (its call to a copy of `rate_damage` on the
//! mirror) against the Rust core, as `rate_damage`'s.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::physics::mirror::{Mirror, Spec};
use crate::physics::rate_damage::{RANGES, SPEC, fill, rate_damage, rate_damage_at, regions};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::rate_current_car", rva(0x4AB5D0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const OWN_SPEC: Spec = (0x4AB5D0, 6, &[0xE8, 0x17, 0x3C, 0x00, 0x00], &[0], 0);

/// The Rust version.
pub unsafe fn rate_current_car() -> Left {
    unsafe { rate_damage() }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { rate_current_car() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(&Mirror::game()), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let m = Mirror::of(&RANGES);
    let inner = match m.copy(SPEC, &|_| None) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let target = inner.addr();
    let copy = match m.copy(OWN_SPEC, &|a| (a == 0x4AF1EC).then_some(target)) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = m.regions();
    for seed in 11..=16u32 {
        let saved = capture(&all);
        fill(&m, seed);
        let input = capture(&all);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        restore(&all, &input);
        let left = unsafe { rate_damage_at(&m) };
        let mine = capture(&all);
        let regs_ok = left.matches(&Regs::SENTINEL, &ro);
        t.check(regs_ok && theirs == mine, || {
            format!("seed {seed}: regs_ok={regs_ok}; the memory differs")
        });
        restore(&all, &saved);
    }
    drop((copy, inner));
    t.finish()
}
