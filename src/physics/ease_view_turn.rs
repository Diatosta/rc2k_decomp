//! Moves the camera's three turns (`camera_heading` and the two after it) each toward its
//! target (`start_heading` and the two after it) by its share (`view_share_heading` and the two
//! after it; all 1 first when `ReplayFrames.step_first` is set): the difference times the
//! share, rounded, is added.
//!
//! Returns the last step.

use crate::math::x87_float::{int, ld, to_int};
use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;

/// The exact version, for `physics::camera_avoid_ground`.
pub fn ease_view_turn(camera: &mut CameraWork, step_first: u32, car: &Car) {
    if step_first != 0 {
        camera.view_share_pitch = 1.0;
        camera.view_share_heading = 1.0;
        camera.view_share_roll = 1.0;
    }
    for (target, turn, share) in [
        (
            camera.start_heading,
            &car.camera_heading,
            camera.view_share_heading,
        ),
        (
            camera.start_pitch,
            &car.camera_pitch,
            camera.view_share_pitch,
        ),
        (
            camera.start_roll,
            &car.camera_roll_turn,
            camera.view_share_roll,
        ),
    ] {
        let step = x87_1(share, target.wrapping_sub(turn.get()));
        turn.set(turn.get().wrapping_add(step));
    }
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`: `int step = int step * share`.
fn x87_1(share: f32, step: u32) -> u32 {
    let t0 = int(step as i32);
    let t1 = ld(share);
    let t2 = t0 * t1;
    to_int(t2) as u32
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87_mem::HostCell;
    use crate::memory::session::Cell32;
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
            plain: |a| {
                let scratch = &crate::test_support::own::scene()
                    .drawing
                    .scene
                    .get()
                    .scratch;
                scratch.set(x87_1(f32::from_bits(HostCell(a[1]).bits()), scratch.get()));
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.ease_view_turn", &blocks(), 1200, 100);
    }
}
