//! The camera's shake, then its step. The shake offsets start at 0; in modes 1 .. 3 with the
//! wheels on the ground, the vertical speed change times `camera_frame_scale`, held within
//! `camera_frame_limit`, becomes `camera_shake_y`, and the shake is a random number / 65536 /
//! `k4096` times `three` times it. Both are 0 again with `still` set. Then
//! `physics::update_camera`.
//!
//! Callers: `physics::car_physics_step`, `drive::finish_run`, `race::show_replay_still`.
//! Refuses what `update_camera` refuses (everything written is put back).

use crate::effects::cell_x87::CellX87;
use crate::errors::Refused;
use crate::math::x87_float::int;
use crate::physics::flow::{hold_high, hold_low, negate};
use crate::physics::step_families::CameraInputs;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;

// Core: `rc2k_core::physics::camera_view::shakes` (the x87 steps stay).
fn steps(c: &CameraInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    scene.lights.state.get().camera_shake_listener.set(0.0);
    scene.lights.state.get().camera_shake_y.set(0);
    if rc2k_core::physics::camera_view::shakes(c.car.camera_view.get(), c.car.all_wheels_off.get())
    {
        x87_1(c.shared, scene);
        hold_high(
            &scene.lights.state.get().camera_shake_y,
            &c.tables.camera_frame_limit,
        );
        negate(&c.tables.camera_frame_limit);
        hold_low(
            &scene.lights.state.get().camera_shake_y,
            &c.tables.camera_frame_limit,
        );
        negate(&c.tables.camera_frame_limit);
        scene
            .drawing
            .scene
            .get()
            .scratch
            .set(c.shared.random.next_random());
        x87_2(c.numbers, scene);
        if c.split_tables.still.get() != 0 {
            scene.lights.state.get().camera_shake_y.set(0);
            scene.lights.state.get().camera_shake_listener.set(0.0);
        }
    }
    crate::physics::update_camera::steps(c, scene)
}

pub fn camera_frame(
    c: &CameraInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    steps(c, scene)
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[camera_shake_y] = [vertical_speed_change] * [camera_frame_scale]`
fn x87_1(shared: &RaceShared, scene: &crate::scene::owner::Scene) {
    let t0 = shared.car().vertical_speed_change.ld();
    let t1 = shared.physics_tables.get().camera_frame_scale.ld();
    let t2 = t0 * t1;
    scene.lights.state.get().camera_shake_y.st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[camera_shake_listener] = (((int [scratch] / [k65536]) / [k4096]) * [three]) *
///  [camera_shake_y]`
fn x87_2(numbers: &Numbers, scene: &crate::scene::owner::Scene) {
    let t0 = int(scene.drawing.scene.get().scratch.get() as i32);
    let t1 = numbers.k65536.ld();
    let t2 = t0 / t1;
    let t3 = numbers.k4096.ld();
    let t4 = t2 / t3;
    let t5 = numbers.three.ld();
    let t6 = t4 * t5;
    let t7 = scene.lights.state.get().camera_shake_y.ld();
    let t8 = t6 * t7;
    scene.lights.state.get().camera_shake_listener.st(t8);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::own::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "camera_frame line 26",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x214098, In::F), (0x31A8E4, In::F)],
                    writes: &[(0x23763C, Out::F)],
                }],
                ulps: 0,
                plain: |_| x87_1(&race().shared, crate::test_support::own::scene()),
            },
            Block {
                name: "camera_frame line 37",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5D40, In::F),
                        (0x1C5D98, In::F),
                        (0x1C5E28, In::F),
                        (0x22400C, In::I),
                        (0x23763C, In::F),
                    ],
                    writes: &[(0x237644, Out::F)],
                }],
                ulps: 0,
                plain: |_| {
                    x87_2(
                        crate::test_support::own::race().course.numbers.get(),
                        crate::test_support::own::scene(),
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.camera_frame", &blocks(), 1200, 100);
    }
}
