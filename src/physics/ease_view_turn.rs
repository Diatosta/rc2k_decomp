//! `physics::ease_view_turn` (0x438B4F): the camera's three turns (`camera_heading`,
//! `71A690`, `71A68C`) each moved toward its target (`start_heading`, `608B58`,
//! `608B54`) by its share (`view_share_heading`, `608BE4`, `608BEC`; all 1 first when
//! `ReplayFrames.step_first` is set): the difference `fild`ed, times the share, `fistp`ed, added.
//!
//! Returns: the last step
//!
//! Caller: `physics::camera_avoid_ground`.

use crate::drive::replay_frames_store::ReplayFrames;
use crate::effects::cell_x87::CellX87;
use crate::math::x87_float::int;
use crate::math::x87_float::to_int;
use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;
use crate::session::Cell32;

/// The exact version, for `physics::camera_avoid_ground`.
pub fn ease_view_turn(
    camera: &CameraWork,
    replay_frames: &ReplayFrames,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) {
    let drawing = &scene.drawing;
    if replay_frames.step_first.get() != 0 {
        for share in [
            &camera.view_share_pitch,
            &camera.view_share_heading,
            &camera.view_share_roll,
        ] {
            share.set_bits(0x3F80_0000);
        }
    }
    for (target, turn, share) in [
        (
            &camera.start_heading,
            &car.camera_heading,
            &camera.view_share_heading,
        ),
        (
            &camera.start_pitch,
            &car.camera_pitch,
            &camera.view_share_pitch,
        ),
        (
            &camera.start_roll,
            &car.camera_roll_turn,
            &camera.view_share_roll,
        ),
    ] {
        drawing
            .scene
            .get()
            .scratch
            .set(target.get().wrapping_sub(turn.get()));
        x87_1(share, scene);
        turn.set(turn.get().wrapping_add(drawing.scene.get().scratch.get()));
    }
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [scratch] = int [scratch] * [s + 0]`
fn x87_1(s: &impl Cell32, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = int(drawing.scene.get().scratch.get() as i32);
    let t1 = s.ld();
    let t2 = t0 * t1;
    drawing.scene.get().scratch.set(to_int(t2) as u32);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::scene::x87_mem::HostCell;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "ease_view_turn line 29",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x22400C, In::I)],
                    writes: &[(0x22400C, Out::I)],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| x87_1(&HostCell(a[1]), crate::scene::owner::scene()),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.ease_view_turn", &blocks(), 1200, 100);
    }
}
