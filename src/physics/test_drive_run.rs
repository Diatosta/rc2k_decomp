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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::flow::{a, base, c0, g, guard, p, x87_run};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, restore, state_entry};

const DRIVE: usize = 0x71A190; // g_carDriveSpeed
const GEAR: usize = 0x71A1AC; // g_carGear
const STEPS: u32 = 0x1388;

/// One step (0x42189D .., in `pusha` / `popa`): `r` the registers there (ebx the step).
fn step(mut r: Regs) -> Result<(), Refused> {
    p(0x71A184, crate::math::x87::loaded(g(DRIVE)));
    unsafe {
        crate::physics::count_down_timers::count_down_timers(&mut r);
        crate::physics::gearbox_step::gearbox_step(&r)?;
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

fn steps(entry: &Regs) -> Result<Left, Refused> {
    for x in [0x5CB97C, 0x5CB980, 0x5CB984, DRIVE, 0x71A194, 0x71A1B0] {
        p(x, 0);
    }
    p(GEAR, 1);
    p(0x71A010, 1);
    let (mut eax, mut edx) = (0, 0);
    for ebx in 0..STEPS {
        step(Regs { ebx, ..*entry })?;
        if g(0x5CB97C) == 0 && !c0(g(DRIVE), g(0x5CB988)) {
            p(0x5CB97C, ebx);
        }
        if g(0x5CB980) == 0 && !c0(g(DRIVE), g(0x5CB98C)) {
            p(0x5CB980, ebx);
        }
        eax = ebx / 20;
        edx = ebx % 20;
        if edx == 0 {
            p(
                0x5CB990 + 4 * eax as usize,
                crate::math::x87::mul(g(DRIVE), g(0x5C5D00)),
            );
            edx = g(GEAR);
            p(0x5CBD90 + 4 * eax as usize, edx);
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
    Ok(Left::nothing()
        .with(EAX, eax)
        .with(EBX, STEPS)
        .with(ECX, 20)
        .with(EDX, edx))
}

/// What the run and its calls write: the work car, the physics' variables, the tables.
pub fn regions() -> Vec<Region> {
    let mut all = crate::physics::car_physics_step::step_regions();
    all.push((a(0x5CB960), 0x5CC1A0 - 0x5CB960));
    all
}

/// The Rust version; everything written put back when it refuses.
pub unsafe fn test_drive_run(entry: &Regs) -> Result<Left, Refused> {
    guard(&regions(), || steps(entry))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { test_drive_run(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// The test of the run and of `physics::test_drive`: from the same state, the drive force
/// as it is and by lot.
pub fn drive_test(
    hook: &RawHook,
    call_original: crate::screens::harness::CallOriginal,
    rust: crate::service::harness::Rust,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let all = regions();
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for case in 0..3u32 {
                let set = || {
                    restore(&all, &saved);
                    if case > 0 {
                        p(0x5CB968, (1.5f32 * case as f32).to_bits());
                    }
                };
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, case {case}"),
                    call_original,
                    Regs::SENTINEL,
                    &set,
                    &all,
                    rust,
                );
            }
        });
    }
    restore(&all, &saved);
    t.finish()
}
