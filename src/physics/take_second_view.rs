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

use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;
use crate::race::view_vars_store::ViewRecord;
use crate::session::Cell32;

/// The exact version, for `physics::update_camera`: `s` the view's record.
pub fn take_second_view(
    camera: &CameraWork,
    car: &Car,
    s: &ViewRecord,
    scene: &crate::scene::owner::Scene,
) {
    x87_1(camera, s.offset_bits(), scene);
    for (x, v) in copies(camera).iter().zip(s.taken()) {
        x.set_bits(v);
    }
    camera.view_b_heading.set(car.car_heading.get());
    let (mut eax, mut ebx) = (car.car_pitch.get(), car.car_roll.get());
    if !matches!(car.camera_blend_view.get(), 1..=3) {
        eax = ((eax as i32) >> 1) as u32;
        ebx = ((ebx as i32) >> 1) as u32;
    }
    camera.view_b_pitch.set(eax);
    camera.view_b_roll.set(ebx);
}

/// Where the record's +0x10 .. +0x38 go.
fn copies(camera: &CameraWork) -> [&'_ dyn Cell32; 11] {
    let w = camera;
    [
        &w.view_b_turn[0],
        &w.view_b_turn[1],
        &w.view_b_turn[2],
        &w.view_b_point[0],
        &w.view_b_point[1],
        &w.view_b_point[2],
        &w.view_b_shares[0],
        &w.view_b_shares[1],
        &w.view_b_shares[2],
        &w.view_b_aux,
        &w.view_b_aux_2,
    ]
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
    let t1 = ld(lights.state.camera_shake_x.bits());
    let t2 = t0 + t1;
    camera.view_b_offset[0].set_bits(st(t2));
    let t3 = ld(offset[1]);
    let t4 = ld(lights.state.camera_shake_y.bits());
    let t5 = t3 + t4;
    camera.view_b_offset[1].set_bits(st(t5));
    let t6 = ld(offset[2]);
    let t7 = ld(lights.state.camera_shake_z.bits());
    let t8 = t6 + t7;
    camera.view_b_offset[2].set_bits(st(t8));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(s, rc) "fld dword ptr [{s} + 0x4]", "fadd dword ptr [{rc} + 0x237638]", "fstp dword ptr [{rc} + 0x208BB0]", "fld dword ptr [{s} + 0x8]", "fadd dword ptr [{rc} + 0x23763C]", "fstp dword ptr [{rc} + 0x208BB4]", "fld dword ptr [{s} + 0xC]", "fadd dword ptr [{rc} + 0x237640]", "fstp dword ptr [{rc} + 0x208BB8]");

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
            reference: |a| unsafe { ref_0(a[0], a[1]) },
            plain: |a| {
                x87_1(
                    &race().physics.camera,
                    [4, 8, 0xC].map(|o| crate::scene::x87_mem::Base::dword(a[0], o)),
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
