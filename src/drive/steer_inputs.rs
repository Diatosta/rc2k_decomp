//! The work car's steering from the input word `input_steer`. Key steering (the word 0, or a
//! car not driven by device, with the keys' bit 0x1000): a step of 0xA00000 x
//! `PhysicsTables.handling[8]` moves `steering_raw` and `steering` toward the side the word says,
//! held at +-0x20000000, or back toward 0 at twice the step. Device steering: the word x
//! 0x20000000 >> 8 x `PhysicsTables.handling[7]`, held +-0x20000000. Then the trim
//! (`steering_efficiency`, negated with `car_flags` bit 0x10) into `steer_trim`, `steer_angle` =
//! `steering` + it, `steer_angle_visual` = `steering_raw` + 4 x it; both 0 unless
//! `steering_damage` is below `hundred` (or unordered).

use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::to_int;
use crate::physics::flow::c0;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;

/// `to` = `from` (signed) x `by`, as a whole number rounded as the x87 rounds.
fn scaled(from: u32, by: f32) -> u32 {
    to_int(int(from as i32) * ld(by.to_bits())) as u32
}

const LEFT: u32 = 0xE000_0000;
const RIGHT: u32 = 0x2000_0000;

fn sign(v: u32) -> bool {
    (v as i32) < 0
}

/// The key step: 0xA00000 x `PhysicsTables.handling[8]`, through `scratch`.
fn key_step(shared: &RaceShared, scene: &crate::scene::owner::Scene) -> u32 {
    scene.draw_work_mut().scratch = 0xA0_0000;
    let step = scaled(
        scene.draw_work().scratch,
        shared.physics_tables().handling[8],
    );
    scene.draw_work_mut().scratch = step;
    step
}

/// One steering value moved by the key step.
fn steer(shared: &RaceShared, cell: &mut u32, step: u32) {
    let word = shared.run().gear_inputs.steer;
    let v = *cell;
    if word != 0 {
        if sign(word) {
            let n = v.wrapping_sub(step);
            *cell = if sign(n.wrapping_sub(LEFT)) { LEFT } else { n };
        } else {
            let n = v.wrapping_add(step);
            *cell = if sign(n.wrapping_sub(RIGHT)) {
                n
            } else {
                RIGHT
            };
        }
        return;
    }
    let step = step.wrapping_add(step);
    if v == 0 {
        return;
    }
    if sign(v) {
        let n = v.wrapping_add(step);
        *cell = if sign(n) { n } else { 0 };
    } else {
        let n = v.wrapping_sub(step);
        *cell = if sign(n) { 0 } else { n };
    }
}

/// (the exact one, for `drive::simple_car_step`, `drive::replay_step` and
/// `race::step_replay_cars`).
pub fn steer_inputs(race: &Race, scene: &crate::scene::owner::Scene) {
    let race_owner = race;
    let course = &race_owner.course;
    let mut keys = race_owner.shared.run().gear_inputs.steer == 0;
    if !keys && race_owner.shared.car().driver_kind != 2 {
        keys = race_owner.shared.car().input_keys & 0x1000 != 0;
    }
    if keys {
        let step = key_step(&race_owner.shared, scene);
        steer(
            &race_owner.shared,
            &mut race_owner.shared.car_mut().steering_raw,
            step,
        );
        let step = key_step(&race_owner.shared, scene);
        steer(
            &race_owner.shared,
            &mut race_owner.shared.car_mut().steering,
            step,
        );
    } else {
        race_owner.shared.car_mut().steering = 0;
        let product =
            i64::from(race_owner.shared.run().gear_inputs.steer as i32) * i64::from(RIGHT as i32);
        scene.draw_work_mut().scratch = (product >> 8) as u32;
        let device = scaled(
            scene.draw_work().scratch,
            race_owner.shared.physics_tables().handling[7],
        );
        scene.draw_work_mut().scratch = device;
        if sign(scene.draw_work().scratch.wrapping_sub(LEFT)) {
            scene.draw_work_mut().scratch = LEFT;
        }
        if !sign(scene.draw_work().scratch.wrapping_sub(RIGHT)) {
            scene.draw_work_mut().scratch = RIGHT;
        }
        race_owner.shared.car_mut().steering_raw = scene.draw_work().scratch;
        race_owner.shared.car_mut().steering = scene.draw_work().scratch;
    }
    // The trim.
    scene.draw_work_mut().scratch = RIGHT;
    let mut car = race_owner.shared.car_mut();
    car.steer_trim = scaled(scene.draw_work().scratch, car.steering_efficiency);
    if car.car_flags & 0x10 != 0 {
        car.steer_trim = car.steer_trim.wrapping_neg();
    }
    let trim = car.steer_trim;
    car.steer_angle = car.steering.wrapping_add(trim);
    car.steer_angle_visual = car.steering_raw.wrapping_add(trim.wrapping_mul(4));
    if !c0(car.steering_damage, course.numbers.hundred.get()) {
        car.steer_angle = 0;
        car.steer_angle_visual = 0;
    }
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::own::race;
    use crate::test_support::own::scene;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "steer_inputs key step",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x22400C, In::I), (0x214C0C, In::F)],
                    writes: &[(0x22400C, Out::I)],
                }],
                ulps: 0,
                plain: |_| {
                    let scratch = scaled(
                        scene().draw_work().scratch,
                        race().shared.physics_tables().handling[8],
                    );
                    scene().draw_work_mut().scratch = scratch;
                },
            },
            Block {
                name: "steer_inputs device",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x22400C, In::I), (0x214C08, In::F)],
                    writes: &[(0x22400C, Out::I)],
                }],
                ulps: 0,
                plain: |_| {
                    let scratch = scaled(
                        scene().draw_work().scratch,
                        race().shared.physics_tables().handling[7],
                    );
                    scene().draw_work_mut().scratch = scratch;
                },
            },
            Block {
                name: "steer_inputs trim",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x22400C, In::I), (0x31A3E0, In::F)],
                    writes: &[(0x31A1E0, Out::I)],
                }],
                ulps: 0,
                plain: |_| {
                    let trim = scaled(
                        scene().draw_work().scratch,
                        race().shared.car().steering_efficiency,
                    );
                    race().shared.car_mut().steer_trim = trim;
                },
            },
        ]
    }

    #[test]
    fn steer_inputs_matches_the_assembly() {
        sparse("drive.steer_inputs", &blocks(), 4000, 203);
    }
}
