//! `physics::free_rev` (0x43EA80): the engine revving free. `physics::engine_torque`;
//! `drive_torque` = `drive_ratio` x `drive_torque_figure` / gear 0's ratio / 10000 x +0x638 (`fild`) /
//! 256; the engine speed + it x the step / the ratio; the drag `scratch` = engine
//! speed x `free_rev_scale` + `free_rev_add`; then the engine speed brought toward 0 by the drag
//! (not past it; at 0 or unordered left alone).
//!
//! The x87 run is plain Rust over `X` (`math/x87_float.rs`) over the game's variables; the
//! original's instructions (from IDA) are `asm_reference` in the tests.
//!
//! Callers: `physics::free_rev_in_neutral`, `physics::free_rev_clutch`.

use crate::effects::cell_x87::CellX87;
use crate::math::x87;
use crate::math::x87_float::int;
use crate::physics::car_store::Car;
use crate::physics::engine_torque::{below, engine_torque};
use crate::physics::gear_store::GEAR;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;

/// 0x43EA85 .. 0x43EAE7, as the original.
fn run(
    numbers: &Numbers,
    physics_tables: &PhysicsTables,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) {
    x87_1(numbers, physics_tables, car, scene);
}

/// The exact version, for `physics::free_rev_in_neutral`, `free_rev_clutch` and their callers.
pub fn free_rev(course: &Course, shared: &RaceShared, scene: &crate::scene::owner::Scene) {
    engine_torque(course, shared, scene);
    run(&course.numbers, &shared.physics_tables, &shared.car, scene);
    let (e, drag) = (
        shared.car.engine_speed.bits(),
        scene.drawing.scene.scratch.get(),
    );
    let x = f32::from_bits(e);
    if x == 0.0 || x.is_nan() {
        return;
    }
    if x < 0.0 {
        let y = x87::add(e, drag);
        shared.car.engine_speed.set(y);
        if !below(y.to_bits(), 0) {
            shared.car.engine_speed.set_bits(0);
        }
    } else {
        let y = x87::sub(e, drag);
        shared.car.engine_speed.set(y);
        if below(y.to_bits(), 0) {
            shared.car.engine_speed.set_bits(0);
        }
    }
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[drive_torque] = (((([drive_ratio] * [drive_torque_figure]) / [rc + 0x2140C0]) / [k10000]) * int [throttle]) / [k256]`
/// `[engine_speed] = (([drive_torque] * [step_time]) / [rc + 0x2140C0]) + [engine_speed]`
/// `[scratch] = ([engine_speed] * [free_rev_scale]) + [free_rev_add]`
fn x87_1(
    numbers: &Numbers,
    physics_tables: &PhysicsTables,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) {
    let t0 = physics_tables.drive_ratio.ld();
    let t1 = GEAR.drive_torque_figure.ld();
    let t2 = t0 * t1;
    let t3 = physics_tables.gear_ratios[1].ld();
    let t4 = t2 / t3;
    let t5 = numbers.k10000.ld();
    let t6 = t4 / t5;
    let t7 = int(car.throttle.bits() as i32);
    let t8 = t6 * t7;
    let t9 = numbers.k256.ld();
    let t10 = t8 / t9;
    physics_tables.drive_torque.st(t10);
    let t11 = physics_tables.drive_torque.ld();
    let t12 = physics_tables.step_time.ld();
    let t13 = t11 * t12;
    let t14 = physics_tables.gear_ratios[1].ld();
    let t15 = t13 / t14;
    let t16 = car.engine_speed.ld();
    let t17 = t15 + t16;
    car.engine_speed.st(t17);
    let t18 = car.engine_speed.ld();
    let t19 = physics_tables.free_rev_scale.ld();
    let t20 = t18 * t19;
    let t21 = physics_tables.free_rev_add.ld();
    let t22 = t20 + t21;
    scene.drawing.scene.scratch.st(t22);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fld dword ptr [{rc} + 0x214740]", "fmul dword ptr [{rc} + 0x1CB968]", "fdiv dword ptr [{rc} + 0x2140C0]", "fdiv dword ptr [{rc} + 0x1C5E34]", "fild dword ptr [{rc} + 0x31A610]", ".byte 0xDE, 0xC9", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fstp dword ptr [{rc} + 0x214750]", "fld dword ptr [{rc} + 0x214750]", "fmul dword ptr [{rc} + 0x21388C]", "fdiv dword ptr [{rc} + 0x2140C0]", "fadd dword ptr [{rc} + 0x31A1B0]", "fstp dword ptr [{rc} + 0x31A1B0]", "fld dword ptr [{rc} + 0x31A1B0]", "fmul dword ptr [{rc} + 0x213FD8]", "fadd dword ptr [{rc} + 0x213FD4]", "fstp dword ptr [{rc} + 0x22400C]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "free_rev line 20",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5E2C, In::F),
                    (0x1C5E34, In::F),
                    (0x1CB968, In::F),
                    (0x21388C, In::F),
                    (0x213FD4, In::F),
                    (0x213FD8, In::F),
                    (0x2140C0, In::F),
                    (0x214740, In::F),
                    (0x214750, In::F),
                    (0x31A1B0, In::F),
                    (0x31A610, In::I),
                ],
                writes: &[(0x214750, Out::F), (0x22400C, Out::F), (0x31A1B0, Out::F)],
            }],
            ulps: 0,
            reference: |a| unsafe { ref_0(a[0]) },
            plain: |_| {
                x87_1(
                    &race().course.numbers,
                    &race().shared.physics_tables,
                    &race().shared.car,
                    crate::scene::owner::scene(),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
