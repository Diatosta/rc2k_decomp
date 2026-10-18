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
use crate::service::harness::dispatch_state;
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

enum To {
    Up,
    DownFrom,
    Neutral,
    Start,
    Reverse,
    Done,
}

/// Up (0x43C66D).
fn up(r: &mut Regs) -> Result<To, Refused> {
    r.ebx = g(GEAR);
    if r.ebx == g(GEAR_COUNT) {
        return Ok(To::Done);
    }
    if r.ebx == u32::MAX {
        return Ok(To::Start);
    }
    wheels(r)?;
    p(GEAR, g(GEAR).wrapping_add(1));
    revs(r)?;
    settle();
    Ok(To::Done)
}

/// Down (0x43C6B7).
fn down(r: &mut Regs) -> Result<To, Refused> {
    wheels(r)?;
    p(GEAR, g(GEAR).wrapping_sub(1));
    revs(r)?;
    if below(g(ENGINE), g(0x614104)) {
        settle();
        return Ok(To::Done);
    }
    p(GEAR, g(GEAR).wrapping_add(1));
    revs(r)?;
    Ok(To::Neutral)
}

/// The automatic's tests (0x43C798).
fn tests(r: &mut Regs) -> Result<To, Refused> {
    r.ebx = g(GEAR);
    let kept = g(ENGINE);
    revs_speed(r)?;
    p(SCRATCH, over_top());
    p(ENGINE, kept);
    if (r.ebx.wrapping_sub(1) as i32) >= 0 && g(0x71A22C) == 0 && !below(g(SCRATCH), g(0x5CB958)) {
        return Ok(To::Up);
    }
    if r.ebx == u32::MAX && g(0x71A614) != 0 && !below(g(SPEED), g(0x613E98)) {
        return Ok(To::Start);
    }
    p(SCRATCH, over_top());
    if (r.ebx.wrapping_sub(2) as i32) >= 0 {
        let (rb, rl) = (ratio(r.ebx)?, ratio(r.ebx.wrapping_sub(1))?);
        if mul_div_status(g(SCRATCH), rb, rl, g(0x5CB964)) & 0x100 != 0 {
            let point = if g(0x71A614) != 0 { 0x5CB960 } else { 0x5CB95C };
            if below(g(SCRATCH), g(point)) {
                return Ok(To::DownFrom);
            }
        }
    }
    if r.ebx == 1 && g(0x71A614) != 0 && below(g(SPEED), g(0x613E94)) {
        return Ok(To::Reverse);
    }
    Ok(To::Done)
}

/// The steps on `r`.
pub fn auto_gearbox_in(r: &mut Regs) -> Result<(), Refused> {
    p(CHANGED, 0);
    let mut to = 'first: {
        if bit(r, g(GEAR))? {
            break 'first To::Neutral;
        }
        if g(GEAR) != 0 && below(g(ENGINE), g(0x6140F8)) {
            break 'first To::Neutral;
        }
        if g(GEAR) == 0 {
            if mul_status(g(LIMIT), g(0x613F18), g(ENGINE)) & 0x100 == 0 {
                return Ok(());
            }
            break 'first if g(0x6138A8) == u32::MAX {
                To::Reverse
            } else {
                To::Start
            };
        }
        if g(0x71A408) != 0 {
            break 'first To::Neutral;
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
            To::Up => up(r)?,
            To::DownFrom => {
                r.ebx = g(GEAR);
                down(r)?
            }
            To::Neutral => {
                neutral();
                To::Done
            }
            To::Start => {
                pick_start_gear_in(r)?;
                To::Done
            }
            To::Reverse => {
                reverse(r, false)?;
                To::Done
            }
            To::Done => return Ok(()),
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

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_gearbox(&mut t, call_original, &rust, &|_| {});
    t.finish()
}
