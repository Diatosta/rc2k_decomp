//! The work car's steering from the input word `input_steer`. Key steering (the word 0, or a
//! car not driven by device, with the keys' bit 0x1000): a step of 0xA00000 x
//! `PhysicsTables.handling[8]` moves `steering_raw` and `steering` toward the side the word says,
//! held at +-0x20000000, or back toward 0 at twice the step. Device steering: the word x
//! 0x20000000 >> 8 x `PhysicsTables.handling[7]`, held +-0x20000000. Then the trim
//! (`steering_efficiency`, negated with `car_flags` bit 0x10) into `steer_trim`, `steer_angle` =
//! `steering` + it, `steer_angle_visual` = `steering_raw` + 4 x it; both 0 unless
//! `steering_damage` is below `hundred` (or unordered).

use crate::effects::cell_x87::CellX87;
use crate::math::x87_float::int;
use crate::math::x87_float::to_int;
use crate::memory::session::{DwordLike, FloatLike};
use crate::physics::flow::c0;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;

/// `to` = `from` (signed) x `by`, as a whole number rounded as the x87 rounds.
fn scale(from: &impl DwordLike, by: &impl FloatLike, to: &impl DwordLike) {
    to.set(to_int(int(from.get() as i32) * by.ld()) as u32);
}

const LEFT: u32 = 0xE000_0000;
const RIGHT: u32 = 0x2000_0000;

fn sign(v: u32) -> bool {
    (v as i32) < 0
}

/// The key step: 0xA00000 x `PhysicsTables.handling[8]`, through `scratch`.
fn key_step(shared: &RaceShared, scene: &crate::scene::owner::Scene) -> u32 {
    let drawing = &scene.drawing;
    drawing.scene.get().scratch.set(0xA0_0000);
    scale(
        &drawing.scene.get().scratch,
        &shared.physics_tables.get().handling[8],
        &drawing.scene.get().scratch,
    );
    drawing.scene.get().scratch.get()
}

/// One steering value moved by the key step.
fn steer(shared: &RaceShared, cell: &impl DwordLike, step: u32) {
    let word = shared.physics_tables.get().input_steer.get();
    let v = cell.get();
    if word != 0 {
        if sign(word) {
            let n = v.wrapping_sub(step);
            cell.set(if sign(n.wrapping_sub(LEFT)) { LEFT } else { n });
        } else {
            let n = v.wrapping_add(step);
            cell.set(if sign(n.wrapping_sub(RIGHT)) {
                n
            } else {
                RIGHT
            });
        }
        return;
    }
    let step = step.wrapping_add(step);
    if v == 0 {
        return;
    }
    if sign(v) {
        let n = v.wrapping_add(step);
        cell.set(if sign(n) { n } else { 0 });
    } else {
        let n = v.wrapping_sub(step);
        cell.set(if sign(n) { 0 } else { n });
    }
}

/// (the exact one, for `drive::simple_car_step`, `drive::replay_step` and
/// `race::step_replay_cars`).
pub fn steer_inputs(race: &Race, scene: &crate::scene::owner::Scene) {
    let race_owner = race;
    let drawing = &scene.drawing;
    let course = &race_owner.course;
    let mut keys = race_owner.shared.physics_tables.get().input_steer.get() == 0;
    if !keys && race_owner.shared.car().driver_kind.get() != 2 {
        keys = race_owner.shared.car().input_keys.get() & 0x1000 != 0;
    }
    if keys {
        steer(
            &race_owner.shared,
            &race_owner.shared.car().steering_raw,
            key_step(&race_owner.shared, scene),
        );
        steer(
            &race_owner.shared,
            &race_owner.shared.car().steering,
            key_step(&race_owner.shared, scene),
        );
    } else {
        race_owner.shared.car().steering.set(0);
        let product = i64::from(race_owner.shared.physics_tables.get().input_steer.get() as i32)
            * i64::from(RIGHT as i32);
        drawing.scene.get().scratch.set((product >> 8) as u32);
        scale(
            &drawing.scene.get().scratch,
            &race_owner.shared.physics_tables.get().handling[7],
            &drawing.scene.get().scratch,
        );
        if sign(drawing.scene.get().scratch.get().wrapping_sub(LEFT)) {
            drawing.scene.get().scratch.set(LEFT);
        }
        if !sign(drawing.scene.get().scratch.get().wrapping_sub(RIGHT)) {
            drawing.scene.get().scratch.set(RIGHT);
        }
        race_owner
            .shared
            .car()
            .steering_raw
            .set(drawing.scene.get().scratch.get());
        race_owner
            .shared
            .car()
            .steering
            .set(drawing.scene.get().scratch.get());
    }
    // The trim.
    drawing.scene.get().scratch.set(RIGHT);
    scale(
        &drawing.scene.get().scratch,
        &race_owner.shared.car().steering_efficiency,
        &race_owner.shared.car().steer_trim,
    );
    if race_owner.shared.car().car_flags.get() & 0x10 != 0 {
        race_owner
            .shared
            .car()
            .steer_trim
            .set(race_owner.shared.car().steer_trim.get().wrapping_neg());
    }
    let trim = race_owner.shared.car().steer_trim.get();
    race_owner
        .shared
        .car()
        .steer_angle
        .set(race_owner.shared.car().steering.get().wrapping_add(trim));
    race_owner.shared.car().steer_angle_visual.set(
        race_owner
            .shared
            .car()
            .steering_raw
            .get()
            .wrapping_add(trim.wrapping_mul(4)),
    );
    if !c0(
        race_owner.shared.car().steering_damage.get(),
        course.numbers.get().hundred.get(),
    ) {
        race_owner.shared.car().steer_angle.set(0);
        race_owner.shared.car().steer_angle_visual.set(0);
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
                    let drawing = &scene().drawing;
                    scale(
                        &drawing.scene.get().scratch,
                        &race().shared.physics_tables.get().handling[8],
                        &drawing.scene.get().scratch,
                    );
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
                    let drawing = &scene().drawing;
                    scale(
                        &drawing.scene.get().scratch,
                        &race().shared.physics_tables.get().handling[7],
                        &drawing.scene.get().scratch,
                    );
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
                    let drawing = &scene().drawing;
                    scale(
                        &drawing.scene.get().scratch,
                        &race().shared.car().steering_efficiency,
                        &race().shared.car().steer_trim,
                    );
                },
            },
        ]
    }

    #[test]
    fn steer_inputs_matches_the_assembly() {
        sparse("drive.steer_inputs", &blocks(), 4000, 203);
    }
}
