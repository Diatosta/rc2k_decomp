//! `physics::make_gear_ratios` (0x4219E6): the car's gear ratios (`g_gearRatios`, by gear,
//! `physics/gearing.rs`) from its setup: the top gear's (`g_gearRatios[g_gearCount]`) and
//! the first's (`flt_6140C4`) each lo + (hi - lo) `flt_5C5DA4` `dword_71A000` (the
//! setup's gearing, a whole number) over `flt_5C5D00` and `flt_71A1B4` (hi / lo
//! `flt_5CB978` / `flt_5CB970` and `flt_5CB974` / `flt_5CB96C`); the gears between in
//! even steps ((top - first) / (count - 1), kept at 80 bits on the stack and added on:
//! gears 2 .. 5 always, 6 and 7 one more step each only past five and six gears; the
//! step itself rounded into `dword_62400C`); reverse (`flt_6140BC`) the first's negative;
//! neutral (`g_gearRatios[0]`) `flt_613FE8`.
//!
//! In:  nothing
//! Out: ebx `g_gearCount`; the rest kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B63F) and `sub_4217E4` (the car specification's). Refuses
//! (before anything) a gear count above 7 (past `g_gearRatios`).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_run::{ext_add, gear_step_80, lerp_int_over2};
use crate::physics::car_store::CAR;
use crate::physics::gear_store::GEAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner_course::Course;
use crate::scene::matrix3::{extended, rounded};
use crate::session::Cell32;

const RATIO: usize = 8; // index of gear 0 in `game()`'s list

/// The Rust core on the variables' cells `v`: the gear count.
pub fn ratios_at(course: &Course, v: &[&dyn Cell32]) -> Result<u32, Refused> {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    let n = get(0);
    if n > 7 {
        return Err(BadInput("a gear count above 7 (past g_gearRatios)").into());
    }
    let (k, d) = (
        course.numbers.quarter.bits(),
        course.numbers.two_and_a_quarter.bits(),
    );
    let one: u32 = course.numbers.one.bits();
    let i = get(5) as i32;
    put(
        RATIO + n as usize,
        lerp_int_over2(get(1), get(2), k, i, d, get(6)),
    );
    put(RATIO + 1, lerp_int_over2(get(3), get(4), k, i, d, get(6)));
    let step = gear_step_80(get(RATIO + n as usize), get(RATIO + 1), n as i32, one);
    let mut acc = extended(get(RATIO + 1));
    for gear in 2..=5 {
        acc = ext_add(acc, step);
        put(RATIO + gear, rounded(acc));
    }
    if n > 5 {
        acc = ext_add(acc, step);
    }
    put(RATIO + 6, rounded(acc));
    if n > 6 {
        acc = ext_add(acc, step);
    }
    put(RATIO + 7, rounded(acc));
    put(16, rounded(step));
    put(7, x87::negated(get(RATIO + 1)));
    put(RATIO, x87::loaded(get(17)));
    Ok(n)
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &PHYSICS.gear_count as &dyn Cell32,
        &GEAR.figures[3],
        &GEAR.figures[1],
        &GEAR.figures[2],
        &GEAR.figures[0],
        &CAR.gear_ratio_setting,
        &CAR.engine_speed_limit,
        &PHYSICS.gear_ratios[0],
        &PHYSICS.gear_ratios[1],
        &PHYSICS.gear_ratios[2],
        &PHYSICS.gear_ratios[3],
        &PHYSICS.gear_ratios[4],
        &PHYSICS.gear_ratios[5],
        &PHYSICS.gear_ratios[6],
        &PHYSICS.gear_ratios[7],
        &PHYSICS.gear_ratios[8],
        &crate::scene::owner::scene().drawing.scene.scratch,
        &PHYSICS.gear_ratio_step,
    ]
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::test_drive`): the gear count.
pub fn make_gear_ratios(course: &Course) -> Result<u32, Refused> {
    ratios_at(course, &game())
}
