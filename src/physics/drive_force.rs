//! `physics::drive_force` (0x43CB14): the engine's drive. `physics::engine_torque_or_cut`;
//! `flt_614740` = (torque + `flt_614744`) x +0x638 (`unk_71A610`, `fild`) / 256 -
//! `flt_614744`; `flt_614750` = it x `flt_5CB968` / the gear's ratio / 10000, or 0 with
//! +0x1D0 (`unk_71A1A8`) set; `physics::drive_torque_share`.
//!
//! In:  nothing
//! Out: ebx the gear, and what the calls leave; no flags read (the caller goes on with a
//!      call)
//!
//! Caller: `sub_43B635` (0x43B732), every physics step.
//! Refuses (before anything) a gear outside -1 ..= 7.
//! Self-test: `physics::engine_torque`'s (`test_engine_floats`), the converted version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::EBX;
use crate::math::x87_run::{mul_div2, sum_times_int_over_minus};
use crate::physics::drive_torque_share::drive_torque_share;
use crate::physics::engine_torque::{g, p};
use crate::physics::engine_torque_or_cut::engine_torque_or_cut;
use crate::screens::harness::Left;

/// The Rust version (the exact one, for the Rust callers' exact comparisons).
pub unsafe fn drive_force() -> Result<Left, Refused> {
    let gear = g(0x71A1AC);
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    let left = unsafe { engine_torque_or_cut() };
    p(
        0x614740,
        sum_times_int_over_minus(g(0x614748), g(0x614744), g(0x71A610) as i32, g(0x5C5E2C)),
    );
    p(
        0x614750,
        mul_div2(
            g(0x614740),
            g(0x5CB968),
            g(0x6140C0 + 4 * gear as usize),
            g(0x5C5E34),
        ),
    );
    if g(0x71A1A8) != 0 {
        p(0x614750, 0);
    }
    let eax = unsafe { drive_torque_share() };
    Ok(left.with(EBX, gear).with(crate::framework::EAX, eax))
}
