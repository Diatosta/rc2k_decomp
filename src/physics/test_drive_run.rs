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
//! Caller: `physics::test_drive`. Refuses a gear outside -1 ..= 7 (past `g_gearRatios`) and
//! what its calls refuse; everything written put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics::car_store::CAR;
use crate::physics::flow::c0;
use crate::physics::gear_store::GEAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner_course::Course;
use crate::session::Float;

pub const STEPS: u32 = 0x1388;

/// One step (0x42189D .., in `pusha` / `popa`).
fn step(course: &Course) -> Result<(), Refused> {
    CAR.local_velocity[2].set_bits(crate::math::x87::loaded(CAR.drive_speed_a.bits()));
    crate::physics::count_down_timers::count_down_timers();
    crate::physics::gearbox_step::gearbox_step(course)?;
    crate::physics::engine_torque::engine_torque();
    let gear = CAR.gear.get();
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    x87_1(course, &PHYSICS.gear_ratios[gear.wrapping_add(1) as usize]);
    crate::physics::drive_torque_share::drive_torque_share();
    x87_2(course);
    { crate::physics::revs_from_wheels::revs_from_wheels() }?;
    Ok(())
}

fn steps(course: &Course) -> Result<(), Refused> {
    for mark in &GEAR.test_marks {
        mark.set(0);
    }
    CAR.drive_speed_a.set_bits(0);
    CAR.drive_speed_b.set_bits(0);
    CAR.engine_speed.set_bits(0);
    CAR.gear.set(1);
    CAR.gearbox_kind.set(1);
    for n in 0..STEPS {
        step(course)?;
        if GEAR.test_marks[0].get() == 0
            && !c0(CAR.drive_speed_a.bits(), GEAR.test_limits[0].bits())
        {
            GEAR.test_marks[0].set(n);
        }
        if GEAR.test_marks[1].get() == 0
            && !c0(CAR.drive_speed_a.bits(), GEAR.test_limits[1].bits())
        {
            GEAR.test_marks[1].set(n);
        }
        if n % 20 == 0 {
            let slot = (n / 20) as usize;
            GEAR.test_speeds[slot].set_bits(crate::math::x87::mul(
                CAR.drive_speed_a.bits(),
                course.numbers.two_and_a_quarter.bits(),
            ));
            GEAR.test_gears[slot].set(CAR.gear.get());
        }
    }
    x87_3(course);
    Ok(())
}

/// The Rust version.
pub fn test_drive_run(course: &Course) -> Result<(), Refused> {
    steps(course)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214750] = (([rc + 0x214748] * [rc + 0x1CB968]) / [r + 0]) / [rc + 0x1C5E34]`
fn x87_1(course: &Course, r: &Float) {
    let t0 = ld(PHYSICS.engine_force.bits());
    let t1 = ld(GEAR.drive_torque_figure.bits());
    let t2 = t0 * t1;
    let t3 = ld(r.bits());
    let t4 = t2 / t3;
    let t5 = ld(course.numbers.k10000.bits());
    let t6 = t4 / t5;
    PHYSICS.drive_torque.set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x22400C] = -(((([rc + 0x31A190] * [rc + 0x31A190]) * [rc + 0x213FCC]) / [rc + 0x1C5E34]) + [rc + 0x213FD0])`
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_2(course: &Course) {
    let t0 = ld(CAR.drive_speed_a.bits());
    let t1 = t0 * t0;
    let t2 = ld(PHYSICS.air_drag.bits());
    let t3 = t1 * t2;
    let t4 = ld(course.numbers.k10000.bits());
    let t5 = t3 / t4;
    let t6 = ld(PHYSICS.rolling_drag.bits());
    let t7 = t5 + t6;
    let t8 = -t7;
    crate::scene::owner::scene()
        .drawing
        .scene
        .scratch
        .set_bits(st(t8));
    let t9 = ld(PHYSICS.drive_torque.bits());
    let t10 = t8 + t9;
    let t11 = ld(PHYSICS.step_time.bits());
    let t12 = t10 * t11;
    let t13 = ld(CAR.drive_speed_a.bits());
    let t14 = t12 + t13;
    CAR.drive_speed_a.set_bits(st(t14));
    CAR.drive_speed_b.set_bits(st(t14));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x1CB984] = [rc + 0x31A190] * [rc + 0x1C5D00]`
/// `[rc + 0x1CB97C] = int [rc + 0x1CB97C] / [rc + 0x1C5D14]`
/// `[rc + 0x1CB980] = int [rc + 0x1CB980] / [rc + 0x1C5D14]`
fn x87_3(course: &Course) {
    let t0 = ld(CAR.drive_speed_a.bits());
    let t1 = ld(course.numbers.two_and_a_quarter.bits());
    let t2 = t0 * t1;
    GEAR.test_marks[2].set_bits(st(t2));
    let t3 = int(GEAR.test_marks[0].bits() as i32);
    let t4 = ld(course.numbers.hundred.bits());
    let t5 = t3 / t4;
    GEAR.test_marks[0].set_bits(st(t5));
    let t6 = int(GEAR.test_marks[1].bits() as i32);
    let t7 = ld(course.numbers.hundred.bits());
    let t8 = t6 / t7;
    GEAR.test_marks[1].set_bits(st(t8));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::view;

    // The original's instructions, one function per run.
    reference!(ref_0(rc, r) "fld dword ptr [{rc} + 0x214748]", "fmul dword ptr [{rc} + 0x1CB968]", "fdiv dword ptr [{r}]", "fdiv dword ptr [{rc} + 0x1C5E34]", "fstp dword ptr [{rc} + 0x214750]");
    reference!(ref_1(rc) "fld dword ptr [{rc} + 0x31A190]", ".byte 0xD8, 0xC8", "fmul dword ptr [{rc} + 0x213FCC]", "fdiv dword ptr [{rc} + 0x1C5E34]", "fadd dword ptr [{rc} + 0x213FD0]", ".byte 0xD9, 0xE0", "fst dword ptr [{rc} + 0x22400C]", "fadd dword ptr [{rc} + 0x214750]", "fmul dword ptr [{rc} + 0x21388C]", "fadd dword ptr [{rc} + 0x31A190]", "fst dword ptr [{rc} + 0x31A190]", "fstp dword ptr [{rc} + 0x31A194]");
    reference!(ref_2(rc) "fld dword ptr [{rc} + 0x31A190]", "fmul dword ptr [{rc} + 0x1C5D00]", "fstp dword ptr [{rc} + 0x1CB984]", "fild dword ptr [{rc} + 0x1CB97C]", "fdiv dword ptr [{rc} + 0x1C5D14]", "fstp dword ptr [{rc} + 0x1CB97C]", "fild dword ptr [{rc} + 0x1CB980]", "fdiv dword ptr [{rc} + 0x1C5D14]", "fstp dword ptr [{rc} + 0x1CB980]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "test_drive_run line 42",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x1C5E34, In::F), (0x1CB968, In::F), (0x214748, In::F)],
                        writes: &[(0x214750, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| x87_1(&crate::race::owner::race().course, view(a[1], 0)),
            },
            Block {
                name: "test_drive_run line 51",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5E34, In::F),
                        (0x21388C, In::F),
                        (0x213FCC, In::F),
                        (0x213FD0, In::F),
                        (0x214750, In::F),
                        (0x31A190, In::F),
                    ],
                    writes: &[(0x22400C, Out::F), (0x31A190, Out::F), (0x31A194, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |_| x87_2(&crate::race::owner::race().course),
            },
            Block {
                name: "test_drive_run line 93",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5D00, In::F),
                        (0x1C5D14, In::F),
                        (0x1CB97C, In::I),
                        (0x1CB980, In::I),
                        (0x31A190, In::F),
                    ],
                    writes: &[(0x1CB97C, Out::F), (0x1CB980, Out::F), (0x1CB984, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0]) },
                plain: |_| x87_3(&crate::race::owner::race().course),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
