//! `physics::take_view_a` (0x437F53): a view's figures (the record at esi) into the
//! first view: +4 .. +0xC plus `dword_637638` .. into `flt_608B98` ..; +0x10 .. +0x24
//! into `flt_608B80` .., `flt_608BC8` ..; +0x28 .. +0x38 into `dword_608BF8`,
//! `608BFC`, `608C00`, `608BF4`, `608C04`; the heading (`unk_71A998`) into
//! `dword_608B5C`; the next two (`unk_71A99C`, `unk_71A9A0`), halved (`sar`) when the
//! mode's record has +0x4C set, into `dword_608B64`, `dword_608B60`.
//!
//! In:  esi the view's record
//! Out: eax, ebx the two, esi the mode's record; the rest kept; no flags read
//!
//! Caller: `sub_437A53`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics::camera_store::CameraWork;
use crate::physics::camera_view::record;
use crate::race::owner::Race;
use crate::race::owner_physics::Physics;
use crate::race::view_vars_store::ViewRecord;

/// The Rust version (the exact one, for `physics::update_camera`): `s` the view's record
/// (esi). The registers it leaves are read by none of its callers.
pub fn take_view_a(
    race: &Race,
    s: &ViewRecord,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    x87_1(&race.physics, s.offset_bits(), scene);
    for (put, v) in copies().iter().zip(s.taken()) {
        put(&race.physics.camera, v);
    }
    race.physics
        .camera
        .view_a_heading
        .set(race.shared.car.car_heading.get());
    let (mut eax, mut ebx) = (
        race.shared.car.car_pitch.get(),
        race.shared.car.car_roll.get(),
    );
    if record(&race.shared)?.halve_tilt.get() != 0 {
        eax = ((eax as i32) >> 1) as u32;
        ebx = ((ebx as i32) >> 1) as u32;
    }
    race.physics.camera.view_a_pitch.set(eax);
    race.physics.camera.view_a_roll.set(ebx);
    Ok(())
}

/// Where the record's +0x10 .. +0x38 go (the statics deref at run time, so a function).
fn copies() -> [fn(&CameraWork, u32); 11] {
    [
        |camera, v| camera.view_a_turn[0].set_bits(v),
        |camera, v| camera.view_a_turn[1].set_bits(v),
        |camera, v| camera.view_a_turn[2].set_bits(v),
        |camera, v| camera.view_a_point[0].set_bits(v),
        |camera, v| camera.view_a_point[1].set_bits(v),
        |camera, v| camera.view_a_point[2].set_bits(v),
        |camera, v| camera.view_a_shares[0].set_bits(v),
        |camera, v| camera.view_a_shares[1].set_bits(v),
        |camera, v| camera.view_a_shares[2].set_bits(v),
        |camera, v| camera.view_a_aux.set(v),
        |camera, v| {
            camera.view_a_aux_2.set(v);
        },
    ]
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (`[s + 4]` .. `[s + 0xC]` the record's offset, passed by value)
/// `[rc + 0x208B98] = [s + 4] + [rc + 0x237638]`
/// `[rc + 0x208B9C] = [s + 8] + [rc + 0x23763C]`
/// `[rc + 0x208BA0] = [s + 0xC] + [rc + 0x237640]`
fn x87_1(physics: &Physics, offset: [u32; 3], scene: &crate::scene::owner::Scene) {
    let lights = &scene.lights;
    let t0 = ld(offset[0]);
    let t1 = ld(lights.state.camera_frame_a.bits());
    let t2 = t0 + t1;
    physics.camera.view_a_offset[0].set_bits(st(t2));
    let t3 = ld(offset[1]);
    let t4 = ld(lights.state.camera_frame_b.bits());
    let t5 = t3 + t4;
    physics.camera.view_a_offset[1].set_bits(st(t5));
    let t6 = ld(offset[2]);
    let t7 = ld(lights.state.camera_frame_c.bits());
    let t8 = t6 + t7;
    physics.camera.view_a_offset[2].set_bits(st(t8));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(s, rc) "fld dword ptr [{s} + 0x4]", "fadd dword ptr [{rc} + 0x237638]", "fstp dword ptr [{rc} + 0x208B98]", "fld dword ptr [{s} + 0x8]", "fadd dword ptr [{rc} + 0x23763C]", "fstp dword ptr [{rc} + 0x208B9C]", "fld dword ptr [{s} + 0xC]", "fadd dword ptr [{rc} + 0x237640]", "fstp dword ptr [{rc} + 0x208BA0]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "take_view_a line 22",
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
            reference: |a| unsafe { ref_0(a[0], a[1]) },
            plain: |a| {
                x87_1(
                    &crate::race::owner::race().physics,
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
