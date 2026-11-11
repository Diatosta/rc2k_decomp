//! `physics::ease_view_blend` (0x4385EB): with the mode's record's blend (+0x58) and
//! change time (+0x3C) not 0: the share done (`physics/camera_view.rs`; 1 less it for a
//! blend below 0) between `view_shift_a` and `view_shift_b` into `camera_zoom`, and between
//! `view_shift_c` and `view_shift_d`, x `k65536`, negated, `fistp`ed into
//! `camera_offset_z`.
//!
//! Caller: `physics::update_camera`.

use crate::math::x87_float::X;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::physics::camera_view::{record, share_done};
use crate::physics::step_families::CameraInputs;
use crate::race::owner_shared::RaceShared;

/// The exact version, for `physics::update_camera`.
pub fn ease_view_blend(
    c: &CameraInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    let esi = record(c.car)?;
    if esi.blend.get() == 0 || esi.change_time.get() == 0 {
        return Ok(());
    }
    share_done(c.shared, esi, scene);
    if (esi.blend.get() as i32) < 0 {
        x87_1(c.shared);
    }
    x87_2(c);
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[view_share] = 1 - [view_share]`
fn x87_1(shared: &RaceShared) {
    let t0 = X(1.0);
    let t1 = ld(shared.tables.view_share.bits());
    let t2 = t0 - t1;
    shared.tables.view_share.set_bits(st(t2));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[camera_zoom] = (([view_shift_b] - [view_shift_a]) * [view_share]) + [view_shift_a]`
/// `int [camera_offset_z] = -(((([view_shift_d] - [view_shift_c]) * [view_share]) + [view_shift_c]) * [k65536])`
fn x87_2(c: &CameraInputs<'_>) {
    let t0 = ld(c.camera.view_shift_b.bits());
    let t1 = ld(c.camera.view_shift_a.bits());
    let t2 = t0 - t1;
    let t3 = ld(c.race_tables.view_share.bits());
    let t4 = t2 * t3;
    let t5 = ld(c.camera.view_shift_a.bits());
    let t6 = t4 + t5;
    c.car.camera_zoom.set_bits(st(t6));
    let t7 = ld(c.camera.view_shift_d.bits());
    let t8 = ld(c.camera.view_shift_c.bits());
    let t9 = t7 - t8;
    let t10 = ld(c.race_tables.view_share.bits());
    let t11 = t9 * t10;
    let t12 = ld(c.camera.view_shift_c.bits());
    let t13 = t11 + t12;
    let t14 = ld(c.numbers.k65536.bits());
    let t15 = t13 * t14;
    let t16 = -t15;
    c.camera.camera_offset_z.set_bits(to_int(t16) as u32);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) ".byte 0xD9, 0xE8", "fsub dword ptr [{rc} + 0x205260]", "fstp dword ptr [{rc} + 0x205260]");
    reference!(ref_1(rc) "fld dword ptr [{rc} + 0x208B24]", "fsub dword ptr [{rc} + 0x208B20]", "fmul dword ptr [{rc} + 0x205260]", "fadd dword ptr [{rc} + 0x208B20]", "fstp dword ptr [{rc} + 0x31A6A0]", "fld dword ptr [{rc} + 0x208B2C]", "fsub dword ptr [{rc} + 0x208B28]", "fmul dword ptr [{rc} + 0x205260]", "fadd dword ptr [{rc} + 0x208B28]", "fmul dword ptr [{rc} + 0x1C5D98]", ".byte 0xD9, 0xE0", "fistp dword ptr [{rc} + 0x208B38]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "ease_view_blend line 27",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x205260, In::F)],
                    writes: &[(0x205260, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0]) },
                plain: |_| x87_1(&race().shared),
            },
            Block {
                name: "ease_view_blend line 34",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5D98, In::F),
                        (0x205260, In::F),
                        (0x208B20, In::F),
                        (0x208B24, In::F),
                        (0x208B28, In::F),
                        (0x208B2C, In::F),
                    ],
                    writes: &[(0x208B38, Out::I), (0x31A6A0, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |_| {
                    x87_2(
                        &crate::physics::step_inputs::StepInputs::of(crate::race::owner::race())
                            .camera(),
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
