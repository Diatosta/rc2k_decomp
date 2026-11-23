//! The engine revving free. `physics::engine_torque`; `drive_torque` = `drive_ratio` x
//! `drive_torque_figure` / gear 0's ratio / 10000 x throttle / 256; the engine speed + it x the
//! step / the ratio; the drag = engine speed x `free_rev_scale` + `free_rev_add`;
//! then the engine speed is brought toward 0 by the drag (not past it; at 0 or unordered left
//! alone).
//!
//! The x87 run is plain Rust over `X` (`math/x87_float.rs`).

use crate::math::cell_x87::CellX87;
use crate::math::cell_x87::PlainX87;
use crate::math::x87;
use crate::math::x87_float::int;
use crate::physics::car_store::Car;
use crate::physics::engine_torque::{below, engine_torque};
use crate::physics::gear_store::GearFigures;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;

/// The drag and the engine speed's step.
fn run(
    numbers: &Numbers,
    physics_tables: &mut PhysicsTables,
    car: &mut Car,
    gear: &GearFigures,
) -> u32 {
    x87_1(numbers, physics_tables, car, gear)
}

/// The exact version, for `physics::free_rev_in_neutral`, `free_rev_clutch` and their callers.
pub fn free_rev(
    course: &Course,
    shared: &RaceShared,
    gear: &GearFigures,
    scene: &crate::scene::owner::Scene,
) {
    engine_torque(course, shared, scene);
    let drag = run(
        &course.numbers,
        &mut shared.physics_tables_mut(),
        &mut shared.car_mut(),
        gear,
    );
    let e = shared.car().engine_speed;
    if e == 0.0 || e.is_nan() {
        return;
    }
    if e < 0.0 {
        let y = x87::add(e, drag);
        shared.car_mut().engine_speed = y;
        if !below(y, 0.0) {
            shared.car_mut().engine_speed = 0.0;
        }
    } else {
        let y = x87::sub(e, drag);
        shared.car_mut().engine_speed = y;
        if below(y, 0.0) {
            shared.car_mut().engine_speed = 0.0;
        }
    }
}

// ---- the x87 arithmetic ----

/// One straight x87 run as plain Rust over `X`; the drag's float bits.
fn x87_1(
    numbers: &Numbers,
    physics_tables: &mut PhysicsTables,
    car: &mut Car,
    gear: &GearFigures,
) -> u32 {
    let t0 = physics_tables.drive_ratio.ld();
    let t1 = gear.drive_torque_figure.ld();
    let t2 = t0 * t1;
    let t3 = physics_tables.gear_ratios[1].ld();
    let t4 = t2 / t3;
    let t5 = numbers.k10000.ld();
    let t6 = t4 / t5;
    let t7 = int(car.throttle as i32);
    let t8 = t6 * t7;
    let t9 = numbers.k256.ld();
    let t10 = t8 / t9;
    physics_tables.drive_torque.st(t10);
    let t11 = physics_tables.drive_torque.ld();
    let t12 = numbers.step_time.ld();
    let t13 = t11 * t12;
    let t14 = physics_tables.gear_ratios[1].ld();
    let t15 = t13 / t14;
    let t16 = car.engine_speed.ld();
    let t17 = t15 + t16;
    car.engine_speed.st(t17);
    let t18 = car.engine_speed.ld();
    let t19 = numbers.free_rev_scale.ld();
    let t20 = t18 * t19;
    let t21 = numbers.free_rev_add.ld();
    let t22 = t20 + t21;
    crate::math::x87_float::st(t22)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::own::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

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
            plain: |_| {
                let drag = x87_1(
                    &crate::test_support::race_places::numbers(),
                    &mut race().shared.physics_tables_mut(),
                    &mut race().shared.car_mut(),
                    race().gear(),
                );
                crate::test_support::own::scene().draw_work_mut().scratch = drag;
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.free_rev", &blocks(), 1200, 100);
    }
}
