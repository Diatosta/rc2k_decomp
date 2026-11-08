//! `physics::ease_view_turn` (0x438B4F): the camera's three turns (`unk_71A688`,
//! `71A690`, `71A68C`) each moved toward its target (`dword_608B50`, `608B58`,
//! `608B54`) by its share (`dword_608BE8`, `608BE4`, `608BEC`; all 1 first when
//! `dword_6EF7CC` is set): the difference `fild`ed, times the share, `fistp`ed, added.
//!
//! Returns: the last step
//!
//! Caller: `physics::camera_avoid_ground`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87_float::to_int;
use crate::math::x87_float::{int, ld};
use crate::race::owner::Race;
use crate::session::Cell32;

/// The Rust version (the exact one, for `physics::camera_avoid_ground`).
pub fn ease_view_turn(race: &Race, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    if race.replay.replay_frames.step_first.get() != 0 {
        for share in [
            &race.physics.camera.view_share_pitch,
            &race.physics.camera.view_share_heading,
            &race.physics.camera.view_share_roll,
        ] {
            share.set_bits(0x3F80_0000);
        }
    }
    for (target, turn, share) in [
        (
            &race.physics.camera.start_heading,
            &race.shared.car.camera_heading,
            &race.physics.camera.view_share_heading,
        ),
        (
            &race.physics.camera.start_pitch,
            &race.shared.car.camera_pitch,
            &race.physics.camera.view_share_pitch,
        ),
        (
            &race.physics.camera.start_roll,
            &race.shared.car.camera_roll_turn,
            &race.physics.camera.view_share_roll,
        ),
    ] {
        drawing
            .scene
            .scratch
            .set(target.get().wrapping_sub(turn.get()));
        x87_1(share, scene);
        turn.set(turn.get().wrapping_add(drawing.scene.scratch.get()));
    }
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x22400C] = int [rc + 0x22400C] * [s + 0]`
fn x87_1(s: &impl Cell32, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = int(drawing.scene.scratch.bits() as i32);
    let t1 = ld(s.bits());
    let t2 = t0 * t1;
    drawing.scene.scratch.set_bits(to_int(t2) as u32);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::HostCell;

    // The original's instructions, one function per run.
    reference!(ref_0(rc, s) "fild dword ptr [{rc} + 0x22400C]", "fmul dword ptr [{s}]", "fistp dword ptr [{rc} + 0x22400C]");

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
            reference: |a| unsafe { ref_0(a[0], a[1]) },
            plain: |a| x87_1(&HostCell(a[1]), crate::scene::owner::scene()),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
