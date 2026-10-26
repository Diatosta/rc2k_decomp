//! `physics::auto_gearbox` (0x43C5C2): the automatic gearbox. +0x238 0. Then:
//! - the gear refused by `physics::gear_bit`, or in gear with the engine speed below
//!   `flt_6140F8`: neutral;
//! - in neutral: unless the engine's top (+0x1DC) x `flt_613F18` is below the engine
//!   speed, nothing; else with `dword_6138A8` -1 into reverse, else
//!   `physics::pick_start_gear`;
//! - +0x430 set: neutral; the hold +0x1CC not 0: it goes down; +0x42C set: nothing;
//! - else (ebx the gear) the engine over its top by `physics::revs_from_speed` (the
//!   engine speed put back): in gear 1 or more with +0x254 clear and it not below
//!   `flt_5CB958`: up (at the top nothing, from reverse `pick_start_gear`, else
//!   `physics::wheels_from_revs`, the gear up, `physics::revs_from_wheels`, settled);
//!   in reverse with +0x63C set and the speed not below `flt_613E98`:
//!   `pick_start_gear`; in gear 2 or more, the share to the gear below under
//!   `flt_5CB964` and the engine over its top under `flt_5CB960` / `flt_5CB95C` (by
//!   +0x63C): down (`wheels_from_revs`, the gear down, `revs_from_wheels`; an engine
//!   speed then not below `flt_614104` takes it back up and ends in neutral, else
//!   settled); in gear 1 with +0x63C set and the speed below `flt_613E94`: into reverse.
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read (the caller jumps on)
//!
//! Caller: `physics::gearbox_step`.
//! Refuses what the calls refuse, and a gear outside -1 ..= 7 where a ratio is read;
//! what was written is put back.
//! Self-test: `physics/gearbox.rs` (`test_gearbox`).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87_run::{mul_div_status, mul_status};
use crate::physics::gearbox::*;
use crate::physics::manual_gearbox::reverse;
use crate::physics::pick_start_gear::pick_start_gear_in;
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use rc2k_core::physics::gear_shifts::{self as shifts, Shift};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::auto_gearbox", rva(0x43C5C2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Up (0x43C66D).
fn up(r: &mut Regs) -> Result<Shift, Refused> {
    r.ebx = g(GEAR);
    if let Some(shift) = shifts::auto_up(r.ebx, g(GEAR_COUNT)) {
        return Ok(shift);
    }
    wheels(r)?;
    p(GEAR, g(GEAR).wrapping_add(1));
    revs(r)?;
    settle();
    Ok(Shift::Done)
}

/// Down (0x43C6B7).
fn down(r: &mut Regs) -> Result<Shift, Refused> {
    wheels(r)?;
    p(GEAR, g(GEAR).wrapping_sub(1));
    revs(r)?;
    if below(g(ENGINE), g(0x614104)) {
        settle();
        return Ok(Shift::Done);
    }
    p(GEAR, g(GEAR).wrapping_add(1));
    revs(r)?;
    Ok(Shift::Neutral)
}

/// The automatic's tests (0x43C798).
fn tests(r: &mut Regs) -> Result<Shift, Refused> {
    r.ebx = g(GEAR);
    let kept = g(ENGINE);
    revs_speed(r)?;
    p(SCRATCH, over_top());
    p(ENGINE, kept);
    let gear = r.ebx;
    shifts::auto_tests(
        gear,
        || g(0x71A22C) != 0,
        || g(0x71A614) != 0,
        || !below(g(SCRATCH), g(0x5CB958)),
        || !below(g(SPEED), g(0x613E98)),
        || {
            p(SCRATCH, over_top());
            if !shifts::above_first(gear) {
                return Ok(false);
            }
            let (rb, rl) = (ratio(gear)?, ratio(gear.wrapping_sub(1))?);
            if mul_div_status(g(SCRATCH), rb, rl, g(0x5CB964)) & 0x100 == 0 {
                return Ok(false);
            }
            let point = if shifts::down_point_clutch(g(0x71A614)) {
                0x5CB960
            } else {
                0x5CB95C
            };
            Ok(below(g(SCRATCH), g(point)))
        },
        || below(g(SPEED), g(0x613E94)),
    )
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::gear_shifts` (the engine's sums and compares stay x87).
pub fn auto_gearbox_in(r: &mut Regs) -> Result<(), Refused> {
    p(CHANGED, 0);
    let mut to = 'first: {
        if bit(r, g(GEAR))? {
            break 'first Shift::Neutral;
        }
        if g(GEAR) != 0 && below(g(ENGINE), g(0x6140F8)) {
            break 'first Shift::Neutral;
        }
        if g(GEAR) == 0 {
            if mul_status(g(LIMIT), g(0x613F18), g(ENGINE)) & 0x100 == 0 {
                return Ok(());
            }
            break 'first if g(0x6138A8) == u32::MAX {
                Shift::Reverse
            } else {
                Shift::Start
            };
        }
        if g(0x71A408) != 0 {
            break 'first Shift::Neutral;
        }
        if g(HOLD) != 0 {
            p(HOLD, g(HOLD).wrapping_sub(1));
            return Ok(());
        }
        if g(0x71A404) != 0 {
            return Ok(());
        }
        tests(r)?
    };
    loop {
        to = match to {
            Shift::Up => up(r)?,
            Shift::Down => {
                r.ebx = g(GEAR);
                down(r)?
            }
            Shift::Neutral => {
                neutral();
                Shift::Done
            }
            Shift::Start => {
                pick_start_gear_in(r)?;
                Shift::Done
            }
            Shift::Reverse => {
                reverse(r, false)?;
                Shift::Done
            }
            Shift::Done => return Ok(()),
        };
    }
}

/// The Rust version.
pub unsafe fn auto_gearbox(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &auto_gearbox_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { auto_gearbox(r) }
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
    test_gearbox(&mut t, call_original, &rust, &|_| {});
    t.finish()
}
