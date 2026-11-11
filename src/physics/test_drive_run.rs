//! `physics::test_drive_run` (0x421848): the car's acceleration measured for its
//! figures: from standing (the drive speeds 0, gear 1, the engine 0, `gearbox_kind` 1), 5000
//! steps of the engine and gearbox alone (in `pusha` / `popa`: the car's speed the drive
//! speed, `physics::count_down_timers`, `physics::gearbox_step`,
//! `physics::engine_torque`, the drive force `engine_force` x `drive_torque_figure` / the gear's ratio
//! / `k10000` into `drive_torque`, `physics::drive_torque_share`, the drag (`fst` into
//! `scratch`) and the drive force step the drive speeds, `physics::revs_from_wheels`);
//! the steps at which the speed first reaches `test_limits` and `flt_5CB98C` into
//! `test_marks`, `dword_5CB980`; every twentieth step the speed x `two_and_a_quarter` and the
//! gear into `test_speeds`[k]`, `test_gears`[k]`; at the end the speed x `two_and_a_quarter` into
//! `dword_5CB984` and the two steps / `hundred` (times) back into theirs.
//!
//! Caller: `physics::test_drive`. Refuses a gear outside -1 ..= 7 (past `g_gearRatios`) and
//! what its calls refuse; everything written put back.

use crate::errors::{BadInput, Refused};
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics::flow::c0;
use crate::physics::gear_store::GEAR;
use crate::race::owner::Race;
use crate::session::Float;

pub const STEPS: u32 = 0x1388;

/// One step (0x42189D ..).
fn step(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    race.shared.car.local_velocity[2].set(crate::math::x87::loaded(&race.shared.car.drive_speed_a));
    crate::physics::count_down_timers::count_down_timers(&race.shared.car, &race.shared.vars);
    crate::physics::gearbox_step::gearbox_step(race, scene)?;
    crate::physics::engine_torque::engine_torque(race, scene);
    let gear = race.shared.car.gear.get();
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    x87_1(
        race,
        &race.shared.physics_tables.gear_ratios[gear.wrapping_add(1) as usize],
    );
    crate::physics::drive_torque_share::drive_torque_share(
        &race.shared.car,
        &race.shared.physics_tables,
        &race.course.numbers,
        scene,
    );
    x87_2(race, scene);
    {
        crate::physics::revs_from_wheels::revs_from_wheels(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.course.numbers,
        )
    }?;
    Ok(())
}

fn steps(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    for mark in &GEAR.test_marks {
        mark.set(0);
    }
    race.shared.car.drive_speed_a.set_bits(0);
    race.shared.car.drive_speed_b.set_bits(0);
    race.shared.car.engine_speed.set_bits(0);
    race.shared.car.gear.set(1);
    race.shared.car.gearbox_kind.set(1);
    for n in 0..STEPS {
        step(race, scene)?;
        if GEAR.test_marks[0].get() == 0
            && !c0(
                race.shared.car.drive_speed_a.bits(),
                GEAR.test_limits[0].bits(),
            )
        {
            GEAR.test_marks[0].set(n);
        }
        if GEAR.test_marks[1].get() == 0
            && !c0(
                race.shared.car.drive_speed_a.bits(),
                GEAR.test_limits[1].bits(),
            )
        {
            GEAR.test_marks[1].set(n);
        }
        if n % 20 == 0 {
            let slot = (n / 20) as usize;
            GEAR.test_speeds[slot].set(crate::math::x87::mul(
                &race.shared.car.drive_speed_a,
                &race.course.numbers.two_and_a_quarter,
            ));
            GEAR.test_gears[slot].set(race.shared.car.gear.get());
        }
    }
    x87_3(race);
    Ok(())
}

pub fn test_drive_run(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    steps(race, scene)
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[drive_torque] = (([engine_force] * [drive_torque_figure]) / [r + 0]) / [k10000]`
fn x87_1(race: &Race, r: &Float) {
    let t0 = ld(race.shared.physics_tables.engine_force.bits());
    let t1 = ld(GEAR.drive_torque_figure.bits());
    let t2 = t0 * t1;
    let t3 = ld(r.bits());
    let t4 = t2 / t3;
    let t5 = ld(race.course.numbers.k10000.bits());
    let t6 = t4 / t5;
    race.shared.physics_tables.drive_torque.set_bits(st(t6));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[scratch] = -(((([drive_speed_a] * [drive_speed_a]) * [air_drag]) / [k10000]) + [rolling_drag])`
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_2(race: &Race, scene: &crate::scene::owner::Scene) {
    let t0 = ld(race.shared.car.drive_speed_a.bits());
    let t1 = t0 * t0;
    let t2 = ld(race.shared.physics_tables.air_drag.bits());
    let t3 = t1 * t2;
    let t4 = ld(race.course.numbers.k10000.bits());
    let t5 = t3 / t4;
    let t6 = ld(race.shared.physics_tables.rolling_drag.bits());
    let t7 = t5 + t6;
    let t8 = -t7;
    scene.drawing.scene.scratch.set_bits(st(t8));
    let t9 = ld(race.shared.physics_tables.drive_torque.bits());
    let t10 = t8 + t9;
    let t11 = ld(race.shared.physics_tables.step_time.bits());
    let t12 = t10 * t11;
    let t13 = ld(race.shared.car.drive_speed_a.bits());
    let t14 = t12 + t13;
    race.shared.car.drive_speed_a.set_bits(st(t14));
    race.shared.car.drive_speed_b.set_bits(st(t14));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x1CB984] = [drive_speed_a] * [two_and_a_quarter]`
/// `[test_marks] = int [test_marks] / [hundred]`
/// `[rc + 0x1CB980] = int [rc + 0x1CB980] / [hundred]`
fn x87_3(race: &Race) {
    let t0 = ld(race.shared.car.drive_speed_a.bits());
    let t1 = ld(race.course.numbers.two_and_a_quarter.bits());
    let t2 = t0 * t1;
    GEAR.test_marks[2].set_bits(st(t2));
    let t3 = int(GEAR.test_marks[0].bits() as i32);
    let t4 = ld(race.course.numbers.hundred.bits());
    let t5 = t3 / t4;
    GEAR.test_marks[0].set_bits(st(t5));
    let t6 = int(GEAR.test_marks[1].bits() as i32);
    let t7 = ld(race.course.numbers.hundred.bits());
    let t8 = t6 / t7;
    GEAR.test_marks[1].set_bits(st(t8));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
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
                plain: |a| x87_1(race(), view(a[1], 0)),
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
                plain: |_| x87_2(race(), crate::scene::owner::scene()),
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
                plain: |_| x87_3(race()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
