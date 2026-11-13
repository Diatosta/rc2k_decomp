//! `physics::take_second_view` (0x438024): `physics::take_first_view` for the second view: the
//! offset plus `camera_shake_x` .. into `view_b_offset`; +0x10 .. +0x24 into `view_b_turn` and
//! `view_b_point`; +0x28 .. +0x38 into `view_b_shares`; the heading into `view_b_heading`; the next
//! two halved (arithmetic shift) unless the mode before (`camera_blend_view`) is 1, 2 or 3, into
//! `view_b_pitch` and `view_b_roll`.
//!
//! Takes: the view's record
//! Returns: the two
//!
//! Caller: `physics::update_camera`.

use crate::effects::cell_x87::CellX87;
use crate::math::x87_float::ld;
use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;
use crate::race::view_vars_store::ViewRecord;

/// The exact version, for `physics::update_camera`: `s` the view's record.
pub fn take_second_view(
    camera: &CameraWork,
    car: &Car,
    s: &ViewRecord,
    scene: &crate::scene::owner::Scene,
) {
    x87_1(camera, s.offset_bits(), scene);
    crate::physics::take_first_view::take_figures(
        s,
        [
            &camera.view_b_turn,
            &camera.view_b_point,
            &camera.view_b_shares,
        ],
        [&camera.view_b_aux, &camera.view_b_aux_2],
    );
    camera.view_b_heading.set(car.car_heading.get());
    let (mut eax, mut ebx) = (car.car_pitch.get(), car.car_roll.get());
    if !matches!(car.camera_blend_view.get(), 1..=3) {
        eax = ((eax as i32) >> 1) as u32;
        ebx = ((ebx as i32) >> 1) as u32;
    }
    camera.view_b_pitch.set(eax as i32);
    camera.view_b_roll.set(ebx as i32);
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (`[s + 4]` .. `[s + 0xC]` the record's offset, passed by value)
/// `[view_b_offset] = [s + 4] + [camera_shake_x]`
/// `[rc + 0x208BB4] = [s + 8] + [camera_shake_y]`
/// `[rc + 0x208BB8] = [s + 0xC] + [camera_shake_z]`
fn x87_1(camera: &CameraWork, offset: [u32; 3], scene: &crate::scene::owner::Scene) {
    let lights = &scene.lights;
    let t0 = ld(offset[0]);
    let t1 = lights.state.get().camera_shake_x.ld();
    let t2 = t0 + t1;
    camera.view_b_offset[0].st(t2);
    let t3 = ld(offset[1]);
    let t4 = lights.state.get().camera_shake_y.ld();
    let t5 = t3 + t4;
    camera.view_b_offset[1].st(t5);
    let t6 = ld(offset[2]);
    let t7 = lights.state.get().camera_shake_z.ld();
    let t8 = t6 + t7;
    camera.view_b_offset[2].st(t8);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "take_second_view line 22",
            operands: vec![
                Operand::Buffer {
                    reads: &[(4, In::F), (8, In::F), (0xC, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x237638, In::F), (0x23763C, In::F), (0x237640, In::F)],
                    writes: &[(0x208BB0, Out::F), (0x208BB4, Out::F), (0x208BB8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                x87_1(
                    race().physics.camera(),
                    [4, 8, 0xC].map(|o| crate::scene::x87_mem::Base::dword(a[0], o)),
                    crate::scene::owner::scene(),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.take_second_view", &blocks(), 1200, 100);
    }
}
