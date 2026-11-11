//! `physics::camera_height_bias` (0x43852D): with the mode's record's +0x54 not 0:
//! `camera_distance` / `camera_bias_limit` held to 0 ..= 1 (1 less it for +0x54 below 0), x
//! `k65536` x `camera_bias_scale` (`fistp`) into `camera_height_bias` and added to the camera's
//! height `unk_71A680`.
//!
//! Caller: `physics::update_camera`.

use crate::math::x87_float::X;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::physics::camera_view::record;
use crate::physics::flow::{ZERO_BITS as ZERO, hold_high, hold_low_bits};
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;

/// The exact version, for `physics::update_camera`.
pub fn camera_height_bias(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    let drawing = &scene.drawing;
    let esi = record(&race.shared.car)?;
    if esi.height_bias.get() == 0 {
        return Ok(());
    }
    x87_1(&race.frame, scene);
    hold_low_bits(&drawing.scene.scratch, ZERO);
    hold_high(&drawing.scene.scratch, &race.course.numbers.one);
    if (esi.height_bias.get() as i32) < 0 {
        x87_2(scene);
    }
    x87_3(race, scene);
    let eax = drawing.scene.scratch.get();
    race.frame
        .view_and_records
        .view_vars
        .camera_height_bias
        .set(eax);
    race.shared.car.camera_position[1]
        .set(race.shared.car.camera_position[1].get().wrapping_add(eax));
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[scratch] = [camera_distance] / [camera_bias_limit]`
fn x87_1(frame: &Frame, scene: &crate::scene::owner::Scene) {
    let t0 = ld(frame.view_and_records.view_vars.camera_distance.bits());
    let t1 = ld(frame.view_and_records.view_vars.camera_bias_limit.bits());
    let t2 = t0 / t1;
    scene.drawing.scene.scratch.set_bits(st(t2));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[scratch] = 1 - [scratch]`
fn x87_2(scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = X(1.0);
    let t1 = ld(drawing.scene.scratch.bits());
    let t2 = t0 - t1;
    drawing.scene.scratch.set_bits(st(t2));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [scratch] = ([scratch] * [k65536]) * [camera_bias_scale]`
fn x87_3(race: &Race, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = ld(drawing.scene.scratch.bits());
    let t1 = ld(race.course.numbers.k65536.bits());
    let t2 = t0 * t1;
    let t3 = ld(race
        .frame
        .view_and_records
        .view_vars
        .camera_bias_scale
        .bits());
    let t4 = t2 * t3;
    drawing.scene.scratch.set_bits(to_int(t4) as u32);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fld dword ptr [{rc} + 0x20631C]", "fdiv dword ptr [{rc} + 0x206290]", "fstp dword ptr [{rc} + 0x22400C]");
    reference!(ref_1(rc) ".byte 0xD9, 0xE8", "fsub dword ptr [{rc} + 0x22400C]", "fstp dword ptr [{rc} + 0x22400C]");
    reference!(ref_2(rc) "fld dword ptr [{rc} + 0x22400C]", "fmul dword ptr [{rc} + 0x1C5D98]", "fmul dword ptr [{rc} + 0x20628C]", "fistp dword ptr [{rc} + 0x22400C]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "camera_height_bias line 24",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x206290, In::F), (0x20631C, In::F)],
                    writes: &[(0x22400C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0]) },
                plain: |_| {
                    x87_1(
                        &crate::race::owner::race().frame,
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "camera_height_bias line 33",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x22400C, In::F)],
                    writes: &[(0x22400C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |_| x87_2(crate::scene::owner::scene()),
            },
            Block {
                name: "camera_height_bias line 40",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F), (0x20628C, In::F), (0x22400C, In::F)],
                    writes: &[(0x22400C, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0]) },
                plain: |_| x87_3(crate::race::owner::race(), crate::scene::owner::scene()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
