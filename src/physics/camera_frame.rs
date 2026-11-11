//! `physics::camera_frame` (0x44FF1B): the camera's shake, then its step. The shake
//! (`flt_637644`) and the offset `dword_637638` .. 0; in modes 1 .. 3 (`unk_71A65C`) with
//! `unk_71A22C` 0: `unk_71A8E4` x `flt_614098` held within ± `flt_614094` (the limit
//! negated and back, `fchs`) into `dword_63763C`, and the shake a random number
//! (`math::next_random`) / 65536 / `flt_5C5E28` x `flt_5C5D40` x it; both 0 again with
//! `dword_601024` set. Then `physics::update_camera`.
//!
//! Callers: `physics::car_physics_step`, `drive::finish_run`, `race::show_replay_still`. Refuses
//! what `update_camera` refuses (everything written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics::flow::{hold_high, hold_low, negate};
use crate::race::numbers_store::Numbers;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;

// Converted: `rc2k_core::physics::camera_view::shakes` (the x87 steps stay).
fn steps(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    scene.lights.state.camera_shake_listener.set_bits(0);
    scene.lights.state.camera_shake_y.set(0);
    if rc2k_core::physics::camera_view::shakes(
        race.shared.car.camera_view.get(),
        race.shared.car.all_wheels_off.get(),
    ) {
        x87_1(&race.shared, scene);
        hold_high(
            &scene.lights.state.camera_shake_y,
            &race.shared.physics_tables.camera_frame_limit,
        );
        negate(&race.shared.physics_tables.camera_frame_limit);
        hold_low(
            &scene.lights.state.camera_shake_y,
            &race.shared.physics_tables.camera_frame_limit,
        );
        negate(&race.shared.physics_tables.camera_frame_limit);
        scene
            .drawing
            .scene
            .scratch
            .set(crate::math::next_random::next_random());
        x87_2(&race.course.numbers, scene);
        if race.physics.split_tables.still.get() != 0 {
            scene.lights.state.camera_shake_y.set(0);
            scene.lights.state.camera_shake_listener.set_bits(0);
        }
    }
    crate::physics::update_camera::steps(race, scene)
}

pub fn camera_frame(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    steps(race, scene)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x23763C] = [rc + 0x31A8E4] * [rc + 0x214098]`
fn x87_1(shared: &RaceShared, scene: &crate::scene::owner::Scene) {
    let t0 = ld(shared.car.vertical_speed_change.bits());
    let t1 = ld(shared.physics_tables.camera_frame_scale.bits());
    let t2 = t0 * t1;
    scene.lights.state.camera_shake_y.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x237644] = (((int [rc + 0x22400C] / [rc + 0x1C5D98]) / [rc + 0x1C5E28]) * [rc + 0x1C5D40]) * [rc + 0x23763C]`
fn x87_2(numbers: &Numbers, scene: &crate::scene::owner::Scene) {
    let t0 = int(scene.drawing.scene.scratch.bits() as i32);
    let t1 = ld(numbers.k65536.bits());
    let t2 = t0 / t1;
    let t3 = ld(numbers.k4096.bits());
    let t4 = t2 / t3;
    let t5 = ld(numbers.three.bits());
    let t6 = t4 * t5;
    let t7 = ld(scene.lights.state.camera_shake_y.bits());
    let t8 = t6 * t7;
    scene.lights.state.camera_shake_listener.set_bits(st(t8));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fld dword ptr [{rc} + 0x31A8E4]", "fmul dword ptr [{rc} + 0x214098]", "fstp dword ptr [{rc} + 0x23763C]");
    reference!(ref_1(rc) "fild dword ptr [{rc} + 0x22400C]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5E28]", "fmul dword ptr [{rc} + 0x1C5D40]", "fmul dword ptr [{rc} + 0x23763C]", "fstp dword ptr [{rc} + 0x237644]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "camera_frame line 26",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x214098, In::F), (0x31A8E4, In::F)],
                    writes: &[(0x23763C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0]) },
                plain: |_| x87_1(&race().shared, crate::scene::owner::scene()),
            },
            Block {
                name: "camera_frame line 37",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5D40, In::F),
                        (0x1C5D98, In::F),
                        (0x1C5E28, In::F),
                        (0x22400C, In::I),
                        (0x23763C, In::F),
                    ],
                    writes: &[(0x237644, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |_| {
                    x87_2(
                        &crate::race::owner::race().course.numbers,
                        crate::scene::owner::scene(),
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
