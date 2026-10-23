//! `physics::grade_or_reset` (0x43BA8E): with `dword_71A234` 2, `physics::reset_current_car`
//! (the work copy's grades reset); else `physics::rate_current_car` (its damage made into
//! grades and failures).
//!
//! In:  nothing
//! Out: as the branch taken leaves them; no flags read
//!
//! Caller: `physics::set_handling` (0x43EB5D), first.
//! Self-test: a copy of the original on a `physics::Mirror` (its calls to copies of the
//! two, and theirs to copies of `reset_car_grades` and `rate_damage`), both branches.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{ESI, Regs};
use crate::physics::mirror::{Mirror, Spec};
use crate::physics::rate_current_car::OWN_SPEC as RATE_CURRENT;
use crate::physics::rate_damage::{SPEC as RATE, rate_damage_at};
use crate::physics::reset_car_grades::{BLOCK_LEN, reset_grades_at};
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};

pub const MODE: u32 = 0x71A234; // dword_71A234: 2 resets the grades
pub const OWN_SPEC: Spec = (
    0x43BA8E,
    0x16,
    &[0x83, 0x3D, 0x34, 0xA2, 0x71, 0x00, 0x02],
    &[9, 0x10],
    1,
);
pub const RESET_CURRENT: Spec = (
    0x43FE18,
    0xC,
    &[0x8D, 0x35, 0xD8, 0x9F, 0x71, 0x00],
    &[6],
    1,
);
pub const RESET_GRADES: Spec = (
    0x440A22,
    0x15A,
    &[0x8D, 0xBE, 0x9C, 0x02, 0x00, 0x00],
    &[],
    2,
);

/// The Rust core on the variables `m` reaches.
pub unsafe fn grade_or_reset_at(m: &Mirror) -> Left {
    unsafe {
        if m.get(MODE) == 2 {
            let car = m.a(0x719FD8);
            reset_grades_at(car, m.a(0x71A00C), m.a(0x71A008)).with(ESI, car)
        } else {
            rate_damage_at(m)
        }
    }
}

/// The Rust version.
pub unsafe fn grade_or_reset() -> Left {
    unsafe { grade_or_reset_at(&Mirror::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { grade_or_reset() })
}

/// What either branch writes.
pub fn regions(m: &Mirror) -> Vec<Region> {
    let mut r = crate::physics::rate_damage::regions(m);
    r.push((m.a(0x719FD8), BLOCK_LEN));
    r
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// The copies of the whole: this one's address and the copies to keep alive.
pub fn copies(m: &Mirror) -> Result<(usize, Vec<crate::redirect::RedirectedCopy>), String> {
    let rate = m.copy(RATE, &|_| None)?;
    let r = rate.addr();
    let rate_current = m.copy(RATE_CURRENT, &|a| (a == 0x4AF1EC).then_some(r))?;
    let grades = m.copy(RESET_GRADES, &|_| None)?;
    let gr = grades.addr();
    let reset = m.copy(RESET_CURRENT, &|a| (a == 0x440A22).then_some(gr))?;
    let (rc, rs) = (rate_current.addr(), reset.addr());
    let own = m.copy(OWN_SPEC, &|a| match a {
        0x4AB5D0 => Some(rc),
        0x43FE18 => Some(rs),
        _ => None,
    })?;
    Ok((own.addr(), vec![own, reset, grades, rate_current, rate]))
}
