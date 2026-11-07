//! `physics::take_view_b` (0x438024): `physics::take_view_a` for the second view: +4 ..
//! +0xC plus `dword_637638` .. into `flt_608BB0` ..; +0x10 .. +0x24 into `flt_608B8C`
//! .., `flt_608BD4` ..; +0x28 .. +0x38 into `dword_608C0C`, `608C10`, `608C14`,
//! `608C08`, `608C18`; the heading into `dword_608B68`; the next two halved (`sar`)
//! unless the mode before (`unk_71A660`) is 1, 2 or 3, into `dword_608B70`,
//! `dword_608B6C`.
//!
//! Takes: the view's record
//! Returns: the two
//!
//! Caller: `sub_437A53`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::race::owner::Race;
use crate::race::owner_physics::Physics;
use crate::race::view_vars_store::ViewRecord;
use crate::session::Cell32;

/// The Rust version (the exact one, for `physics::update_camera`): `s` the view's record
/// (esi). The eax and ebx it leaves are read by none of its callers.
pub fn take_view_b(race: &Race, s: &ViewRecord, scene: &crate::scene::owner::Scene) {
    x87_1(&race.physics, s.offset_bits(), scene);
    for (x, v) in copies(&race.physics).iter().zip(s.taken()) {
        x.set_bits(v);
    }
    race.physics
        .camera
        .view_b_heading
        .set(race.shared.car.car_heading.get());
    let (mut eax, mut ebx) = (
        race.shared.car.car_pitch.get(),
        race.shared.car.car_roll.get(),
    );
    if !matches!(race.shared.car.camera_blend_view.get(), 1..=3) {
        eax = ((eax as i32) >> 1) as u32;
        ebx = ((ebx as i32) >> 1) as u32;
    }
    race.physics.camera.view_b_pitch.set(eax);
    race.physics.camera.view_b_roll.set(ebx);
}

/// Where the record's +0x10 .. +0x38 go.
fn copies(physics: &Physics) -> [&'_ dyn Cell32; 11] {
    let w = &physics.camera;
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

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (`[s + 4]` .. `[s + 0xC]` the record's offset, passed by value)
/// `[rc + 0x208BB0] = [s + 4] + [rc + 0x237638]`
/// `[rc + 0x208BB4] = [s + 8] + [rc + 0x23763C]`
/// `[rc + 0x208BB8] = [s + 0xC] + [rc + 0x237640]`
fn x87_1(physics: &Physics, offset: [u32; 3], scene: &crate::scene::owner::Scene) {
    let lights = &scene.lights;
    let t0 = ld(offset[0]);
    let t1 = ld(lights.state.camera_frame_a.bits());
    let t2 = t0 + t1;
    physics.camera.view_b_offset[0].set_bits(st(t2));
    let t3 = ld(offset[1]);
    let t4 = ld(lights.state.camera_frame_b.bits());
    let t5 = t3 + t4;
    physics.camera.view_b_offset[1].set_bits(st(t5));
    let t6 = ld(offset[2]);
    let t7 = ld(lights.state.camera_frame_c.bits());
    let t8 = t6 + t7;
    physics.camera.view_b_offset[2].set_bits(st(t8));
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
            name: "take_view_b line 22",
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
                    &race().physics,
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
