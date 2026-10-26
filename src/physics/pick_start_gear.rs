//! `physics::pick_start_gear` (0x43C8AC): from neutral, the gear to start in.
//! `physics::copy_car_word_8`; the engine speed kept in `dword_614144`;
//! `physics::revs_from_speed`; `dword_614140` 0 and `flt_61413C` 100.0; then gears 1 .. 6
//! (ebx) in turn: one `physics::gear_bit` does not refuse is set, `revs_from_speed`, the
//! engine speed over its top (+0x1DC) stored; the first below `flt_5CB958` is taken: set,
//! `revs_from_speed`, the engine speed moved back toward the kept one by `flt_613FE0`
//! ((kept - e) x k + e), `physics::wheels_from_revs`, and the change settled (+0x1D0,
//! the hold 20, +0x238 and +0x23C 1). None taken: the last tried stays.
//!
//! In:  nothing
//! Out: the registers as the calls leave them (eax, ebx, ecx); no flags read (the
//!      callers return)
//!
//! Callers: `physics::manual_gearbox`, `physics::auto_gearbox`.
//! Refuses what the calls refuse; what was written is put back.
//! Self-test: `physics/gearbox.rs` (`test_gearbox`), gears made 0 first.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::sub_mul_add;
use crate::physics::copy_car_word_8::copy_car_word_8;
use crate::physics::gearbox::*;
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use rc2k_core::physics::gear_shifts as shifts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::pick_start_gear", rva(0x43C8AC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The steps on `r`.
// Converted: `rc2k_core::physics::gear_shifts::pick_start` (the engine's sums and compares
// stay x87).
pub fn pick_start_gear_in(r: &mut Regs) -> Result<(), Refused> {
    unsafe { copy_car_word_8() }.apply(r);
    p(0x614144, g(ENGINE));
    revs_speed(r)?;
    r.ebx = 1;
    p(0x614140, 0);
    p(0x61413C, x87::loaded(g(0x5C5D14)));
    let taken = shifts::pick_start(|gear| {
        r.ebx = gear;
        if bit(r, gear)? {
            return Ok::<_, Refused>(false);
        }
        p(GEAR, gear);
        revs_speed(r)?;
        p(ENGINE, over_top());
        Ok(below(g(ENGINE), g(0x5CB958)))
    })?;
    let Some(gear) = taken else {
        r.ebx = 7;
        return Ok(());
    };
    r.ebx = gear;
    p(GEAR, r.ebx);
    revs_speed(r)?;
    p(
        ENGINE,
        sub_mul_add(g(0x614144), g(ENGINE), g(0x613FE0), g(ENGINE)),
    );
    wheels(r)?;
    settle();
    Ok(())
}

/// The Rust version.
pub unsafe fn pick_start_gear(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &pick_start_gear_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { pick_start_gear(r) }
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
    test_gearbox(&mut t, call_original, &rust, &|c| c.gear = 0);
    t.finish()
}
