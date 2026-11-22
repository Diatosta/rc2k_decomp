//! `physics::take_first_view` for the second view: the offset plus `camera_shake_x` into
//! `view_b_offset`; the turn and point figures into `view_b_turn` and `view_b_point`; the shares
//! into `view_b_shares`; the heading into `view_b_heading`; the next two halved (arithmetic shift)
//! unless the mode before (`camera_blend_view`) is 1, 2 or 3, into `view_b_pitch` and
//! `view_b_roll`.

use crate::math::x87_float::{ld, stf};
use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;
use crate::race::view_vars_store::ViewRecord;

/// The exact version, for `physics::update_camera`: `s` the view's record.
pub fn take_second_view(
    camera: &mut CameraWork,
    car: &Car,
    s: &ViewRecord,
    scene: &crate::scene::owner::Scene,
) {
    camera.view_b_offset = x87_1(s.offset_bits(), scene);
    crate::physics::take_first_view::take_figures(
        s,
        [
            &mut camera.view_b_turn,
            &mut camera.view_b_point,
            &mut camera.view_b_shares,
        ],
        [&mut camera.view_b_aux, &mut camera.view_b_aux_2],
    );
    camera.view_b_heading = car.car_heading;
    let [pitch, roll] = rc2k_core::physics::camera_view::view_turns(
        [car.car_pitch, car.car_roll],
        !matches!(car.camera_blend_view, 1..=3),
    );
    camera.view_b_pitch = pitch as i32;
    camera.view_b_roll = roll as i32;
}

// ---- the x87 arithmetic ----

/// (`[s + 4]` .. `[s + 0xC]` the record's offset, passed by value) `[view_b_offset] = [s + 4] +
/// [camera_shake_x]`
fn x87_1(offset: [u32; 3], scene: &crate::scene::owner::Scene) -> [f32; 3] {
    let shake = *scene.line_and_shake();
    let t0 = ld(offset[0]);
    let t1 = ld(shake.camera_shake_x);
    let t2 = t0 + t1;
    let o0 = stf(t2);
    let t3 = ld(offset[1]);
    let t4 = ld(shake.camera_shake_y);
    let t5 = t3 + t4;
    let o1 = stf(t5);
    let t6 = ld(offset[2]);
    let t7 = ld(shake.camera_shake_z);
    let t8 = t6 + t7;
    let o2 = stf(t8);
    [o0, o1, o2]
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::own::race;
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
                race().physics.camera_mut().view_b_offset = x87_1(
                    [4, 8, 0xC].map(|o| crate::math::x87_mem::Base::dword(a[0], o)),
                    crate::test_support::own::scene(),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.take_second_view", &blocks(), 1200, 100);
    }
}
