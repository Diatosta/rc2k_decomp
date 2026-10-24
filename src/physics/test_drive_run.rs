//! `physics::test_drive_run` (0x421848): the car's acceleration measured for its
//! figures: from standing (the drive speeds 0, gear 1, the engine 0, `unk_71A010` 1), 5000
//! steps of the engine and gearbox alone (in `pusha` / `popa`: the car's speed the drive
//! speed, `physics::count_down_timers`, `physics::gearbox_step`,
//! `physics::engine_torque`, the drive force `flt_614748` x `flt_5CB968` / the gear's ratio
//! / `flt_5C5E34` into `flt_614750`, `physics::drive_torque_share`, the drag (`fst` into
//! `dword_62400C`) and the drive force step the drive speeds, `physics::revs_from_wheels`);
//! the steps at which the speed first reaches `flt_5CB988` and `flt_5CB98C` into
//! `dword_5CB97C`, `dword_5CB980`; every twentieth step the speed x `flt_5C5D00` and the
//! gear into `flt_5CB990[k]`, `dword_5CBD90[k]`; at the end the speed x `flt_5C5D00` into
//! `dword_5CB984` and the two steps / `flt_5C5D14` (times) back into theirs.
//!
//! In:  nothing
//! Out: ebx 5000, eax the last step / 20, ecx 20, edx its remainder (the gear when 0); the
//!      rest kept; no flags read
//!
//! Caller: `physics::test_drive`.
//! Refuses a gear outside -1 ..= 7 (past `g_gearRatios`) and what its calls refuse;
//! everything written put back.
//! Self-test: the original and the Rust version from the same state (the work car, the
//! physics' variables and the tables saved and put back), the car as it is and with its
//! drive force by lot, under three precisions.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::flow::{a, base, c0, g, p, x87_run};

const DRIVE: usize = 0x71A190; // g_carDriveSpeed
const GEAR: usize = 0x71A1AC; // g_carGear
pub const STEPS: u32 = 0x1388;

/// One step (0x42189D .., in `pusha` / `popa`).
fn step() -> Result<(), Refused> {
    p(0x71A184, crate::math::x87::loaded(g(DRIVE)));
    unsafe {
        crate::physics::count_down_timers::count_down_timers();
        crate::physics::gearbox_step::gearbox_step()?;
        crate::physics::engine_torque::engine_torque();
    }
    let gear = g(GEAR);
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    x87_run!(
        "fld dword ptr [{b} + 0x214748]",
        "fmul dword ptr [{b} + 0x1CB968]",
        "fdiv dword ptr [{r}]",
        "fdiv dword ptr [{b} + 0x1C5E34]",
        "fstp dword ptr [{b} + 0x214750]";
        b = base(), r = a(0x6140C0).wrapping_add(gear.wrapping_mul(4))
    );
    unsafe { crate::physics::drive_torque_share::drive_torque_share() };
    x87_run!(
        "fld dword ptr [{b} + 0x31A190]",
        ".byte 0xD8, 0xC8", // fmul st, st
        "fmul dword ptr [{b} + 0x213FCC]",
        "fdiv dword ptr [{b} + 0x1C5E34]",
        "fadd dword ptr [{b} + 0x213FD0]",
        ".byte 0xD9, 0xE0", // fchs
        "fst dword ptr [{b} + 0x22400C]",
        "fadd dword ptr [{b} + 0x214750]",
        "fmul dword ptr [{b} + 0x21388C]",
        "fadd dword ptr [{b} + 0x31A190]",
        "fst dword ptr [{b} + 0x31A190]",
        "fstp dword ptr [{b} + 0x31A194]";
        b = base()
    );
    unsafe { crate::physics::revs_from_wheels::revs_from_wheels() }?;
    Ok(())
}

fn steps() -> Result<(), Refused> {
    for x in [0x5CB97C, 0x5CB980, 0x5CB984, DRIVE, 0x71A194, 0x71A1B0] {
        p(x, 0);
    }
    p(GEAR, 1);
    p(0x71A010, 1);
    for n in 0..STEPS {
        step()?;
        if g(0x5CB97C) == 0 && !c0(g(DRIVE), g(0x5CB988)) {
            p(0x5CB97C, n);
        }
        if g(0x5CB980) == 0 && !c0(g(DRIVE), g(0x5CB98C)) {
            p(0x5CB980, n);
        }
        if n % 20 == 0 {
            let slot = 4 * (n / 20) as usize;
            p(
                0x5CB990 + slot,
                crate::math::x87::mul(g(DRIVE), g(0x5C5D00)),
            );
            p(0x5CBD90 + slot, g(GEAR));
        }
    }
    x87_run!(
        "fld dword ptr [{b} + 0x31A190]",
        "fmul dword ptr [{b} + 0x1C5D00]",
        "fstp dword ptr [{b} + 0x1CB984]",
        "fild dword ptr [{b} + 0x1CB97C]",
        "fdiv dword ptr [{b} + 0x1C5D14]",
        "fstp dword ptr [{b} + 0x1CB97C]",
        "fild dword ptr [{b} + 0x1CB980]",
        "fdiv dword ptr [{b} + 0x1C5D14]",
        "fstp dword ptr [{b} + 0x1CB980]";
        b = base()
    );
    Ok(())
}

/// The Rust version.
pub fn test_drive_run() -> Result<(), Refused> {
    steps()
}
