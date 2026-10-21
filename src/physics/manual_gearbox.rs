//! `physics::manual_gearbox` (0x43C2AE): the gearbox by the keys (`dword_6138A0` up,
//! `dword_6138A4` down). +0x238 0. Then, in this order (`g_car` fields):
//! - grade mode (+0x25C) 2: the start-line rules (below);
//! - the gear refused by `physics::gear_bit`, or +0x430 set: neutral;
//! - the hold +0x1CC not 0: it goes down, nothing else;
//! - +0x42C set: nothing; +0x38 (`unk_71A010`) set: the start-line rules;
//! - up: from reverse neutral, from neutral `physics::pick_start_gear`, at the top
//!   nothing; else `physics::wheels_from_revs`, `physics::higher_free_gear` (none:
//!   nothing), set, `physics::revs_from_wheels`, settled;
//! - down: from 1 neutral, from neutral into reverse (unless `gear_bit` refuses -1: the
//!   engine speed x 0.25, `wheels_from_revs`, settled), in reverse nothing; else
//!   `wheels_from_revs`, `physics::lower_free_gear` (none: neutral), set,
//!   `revs_from_wheels`; an engine speed then not below `flt_614104` takes the gear back
//!   up (`revs_from_wheels`; the hold 20, +0x238 -1, +0x23C 1), else settled.
//!
//! The start-line rules: down asked: neutral into reverse, a forward gear to neutral; up
//! asked: neutral `pick_start_gear`, reverse to neutral; then, with +0x254 clear, the
//! automatic's tests (in gear 1 or more: the engine over its top by
//! `physics::revs_from_speed`, the engine speed put back, not below `flt_5CB958`: up; in
//! gear 2 or more: down when the next free lower gear's share is below `flt_5CB964` and
//! the engine over its top below `flt_5CB960` / `flt_5CB95C` by +0x63C).
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read (the callers jump on)
//!
//! Callers: `physics::gearbox_step`, the one-call `physics::manual_gearbox_call`.
//! Refuses what the calls refuse, and a gear outside -1 ..= 7 where a ratio is read;
//! what was written is put back.
//! Self-test: `physics/gearbox.rs` (`test_gearbox`).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::mul_div_status;
use crate::physics::gearbox::*;
use crate::physics::higher_free_gear::higher_free_gear;
use crate::physics::lower_free_gear::lower_free_gear;
use crate::physics::pick_start_gear::pick_start_gear_in;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use rc2k_core::physics::gear_shifts::{self as shifts, Shift};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::manual_gearbox", rva(0x43C2AE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Up (0x43C340).
fn up(r: &mut Regs) -> Result<Shift, Refused> {
    r.ebx = g(GEAR);
    if let Some(shift) = shifts::manual_up(r.ebx, || g(GEAR_COUNT)) {
        return Ok(shift);
    }
    wheels(r)?;
    let f = unsafe { higher_free_gear(r.ebx) }?;
    f.apply(r);
    if f.carry {
        return Ok(Shift::Done);
    }
    p(GEAR, r.ecx);
    revs(r)?;
    settle();
    Ok(Shift::Done)
}

/// Down (0x43C379).
fn down(r: &mut Regs) -> Result<Shift, Refused> {
    r.ebx = g(GEAR);
    if let Some(shift) = shifts::manual_down(r.ebx) {
        return Ok(shift);
    }
    wheels(r)?;
    let f = unsafe { lower_free_gear(r.ebx, r.eax) }?;
    f.apply(r);
    if f.carry {
        return Ok(Shift::Neutral);
    }
    p(GEAR, r.ecx);
    revs(r)?;
    if !below(g(ENGINE), g(0x614104)) {
        p(GEAR, g(GEAR).wrapping_add(1));
        revs(r)?;
        p(HOLD, shifts::HOLD);
        p(CHANGED, u32::MAX);
        p(CHANGED_2, 1);
        return Ok(Shift::Done);
    }
    settle();
    Ok(Shift::Done)
}

/// Into reverse (0x43C433; with `quarter` the engine speed x 0.25 first, as here; the
/// automatic's 0x43C74E has no such step).
pub fn reverse(r: &mut Regs, quarter: bool) -> Result<(), Refused> {
    if bit(r, u32::MAX)? {
        return Ok(());
    }
    p(GEAR, u32::MAX);
    if quarter {
        p(ENGINE, x87::mul(g(ENGINE), g(0x5C5DA4)));
    }
    wheels(r)?;
    settle();
    Ok(())
}

/// The start-line rules (0x43C48F).
fn start_line(r: &mut Regs) -> Result<Shift, Refused> {
    r.ebx = g(GEAR);
    if let Some(shift) = shifts::start_line_keys(r.ebx, g(0x6138A4) != 0, || g(0x6138A0) != 0) {
        return Ok(shift);
    }
    if g(0x71A22C) != 0 {
        return Ok(Shift::Done);
    }
    if shifts::forward(r.ebx) {
        let kept = g(ENGINE);
        revs_speed(r)?;
        p(SCRATCH, over_top());
        p(ENGINE, kept);
        if !below(g(SCRATCH), g(0x5CB958)) {
            return Ok(Shift::Up);
        }
    }
    if !shifts::above_first(r.ebx) {
        return Ok(Shift::Done);
    }
    p(SCRATCH, over_top());
    let f = unsafe { lower_free_gear(r.ebx, r.eax) }?;
    f.apply(r);
    if f.carry {
        return Ok(Shift::Done);
    }
    let (rb, rc) = (ratio(r.ebx)?, ratio(r.ecx)?);
    if mul_div_status(g(SCRATCH), rb, rc, g(0x5CB964)) & 0x100 == 0 {
        return Ok(Shift::Done);
    }
    let point = if shifts::down_point_clutch(g(0x71A614)) {
        0x5CB960
    } else {
        0x5CB95C
    };
    if !below(g(SCRATCH), g(point)) {
        return Ok(Shift::Done);
    }
    Ok(Shift::Down)
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::gear_shifts` (the engine's sums and compares stay x87).
pub fn manual_gearbox_in(r: &mut Regs) -> Result<(), Refused> {
    p(CHANGED, 0);
    let mut to = 'first: {
        if g(0x71A234) == 2 {
            break 'first start_line(r)?;
        }
        let gear = g(GEAR);
        if bit(r, gear)? || g(0x71A408) != 0 {
            break 'first Shift::Neutral;
        }
        if g(HOLD) != 0 {
            p(HOLD, g(HOLD).wrapping_sub(1));
            return Ok(());
        }
        if g(0x71A404) != 0 {
            return Ok(());
        }
        if g(0x71A010) != 0 {
            break 'first start_line(r)?;
        }
        r.ebx = g(GEAR);
        shifts::manual_keys(g(0x6138A0) != 0, || g(0x6138A4) != 0)
    };
    loop {
        to = match to {
            Shift::Up => up(r)?,
            Shift::Down => down(r)?,
            Shift::Neutral => {
                neutral();
                Shift::Done
            }
            Shift::Start => {
                pick_start_gear_in(r)?;
                Shift::Done
            }
            Shift::Reverse => {
                reverse(r, true)?;
                Shift::Done
            }
            Shift::Done => return Ok(()),
        };
    }
}

/// The Rust version.
pub unsafe fn manual_gearbox(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &manual_gearbox_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { manual_gearbox(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_gearbox(&mut t, call_original, &rust, &|_| {});
    t.finish()
}
