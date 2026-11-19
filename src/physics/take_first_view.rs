//! Copies a view's figures (a record) into the first view: the offset plus `camera_shake_x` into
//! `view_a_offset`; the turn and point figures into `view_a_turn` and `view_a_point`; the shares
//! into `view_a_shares`; the heading (`car_heading`) into `view_a_heading`; the next two
//! (`car_pitch`, `car_roll`), halved (arithmetic shift) when the mode's record has `halve_tilt`
//! set, into `view_a_pitch` and `view_a_roll`.

use crate::math::cell_x87::CellX87;
use crate::math::x87_float::ld;
use crate::memory::session::{PlainDword, PlainFloat};
use crate::physics::camera_store::CameraWork;
use crate::physics::camera_view::record;
use crate::physics::car_store::Car;
use crate::race::groups_store::ViewAndRecords;
use crate::race::view_vars_store::ViewRecord;

/// The exact version, for `physics::update_camera`: `s` the view's record.
pub fn take_first_view(
    camera: &CameraWork,
    car: &Car,
    view_records: &ViewAndRecords,
    s: &ViewRecord,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    x87_1(camera, s.offset_bits(), scene);
    take_figures(
        s,
        [
            &camera.view_a_turn,
            &camera.view_a_point,
            &camera.view_a_shares,
        ],
        [&camera.view_a_aux, &camera.view_a_aux_2],
    );
    camera.view_a_heading.set(car.car_heading.get());
    let (mut eax, mut ebx) = (car.car_pitch.get(), car.car_roll.get());
    if record(view_records, car)?.halve_tilt.get() != 0 {
        eax = ((eax as i32) >> 1) as u32;
        ebx = ((ebx as i32) >> 1) as u32;
    }
    camera.view_a_pitch.set(eax as i32);
    camera.view_a_roll.set(ebx as i32);
    Ok(())
}

/// The record's turn, point and share figures into a view: three vectors of floats (held in the
/// record's dwords), then two dwords.
pub fn take_figures(s: &ViewRecord, floats: [&[PlainFloat; 3]; 3], dwords: [&PlainDword; 2]) {
    let v = s.taken();
    for (n, cell) in floats.into_iter().flatten().enumerate() {
        cell.set(f32::from_bits(v[n]));
    }
    for (n, cell) in dwords.into_iter().enumerate() {
        cell.set(v[9 + n]);
    }
}

// ---- the x87 arithmetic ----

/// (`[s + 4]` .. `[s + 0xC]` the record's offset, passed by value) `[view_a_offset] = [s + 4] +
/// [camera_shake_x]`
fn x87_1(camera: &CameraWork, offset: [u32; 3], scene: &crate::scene::owner::Scene) {
    let lights = &scene.lights;
    let t0 = ld(offset[0]);
    let t1 = lights.state.get().camera_shake_x.ld();
    let t2 = t0 + t1;
    camera.view_a_offset[0].st(t2);
    let t3 = ld(offset[1]);
    let t4 = lights.state.get().camera_shake_y.ld();
    let t5 = t3 + t4;
    camera.view_a_offset[1].st(t5);
    let t6 = ld(offset[2]);
    let t7 = lights.state.get().camera_shake_z.ld();
    let t8 = t6 + t7;
    camera.view_a_offset[2].st(t8);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "take_first_view line 22",
            operands: vec![
                Operand::Buffer {
                    reads: &[(4, In::F), (8, In::F), (0xC, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x237638, In::F), (0x23763C, In::F), (0x237640, In::F)],
                    writes: &[(0x208B98, Out::F), (0x208B9C, Out::F), (0x208BA0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                x87_1(
                    crate::test_support::own::race().physics.camera(),
                    [4, 8, 0xC].map(|o| crate::math::x87_mem::Base::dword(a[0], o)),
                    crate::test_support::own::scene(),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.take_first_view", &blocks(), 1200, 100);
    }
}
