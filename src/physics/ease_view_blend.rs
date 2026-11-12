//! `physics::ease_view_blend` (0x4385EB): with the mode's record's blend (+0x58) and
//! change time (+0x3C) not 0: the share done (`physics/camera_view.rs`; 1 less it for a
//! blend below 0) between `view_shift_a` and `view_shift_b` into `camera_zoom`, and between
//! `view_shift_c` and `view_shift_d`, x `k65536`, negated, `fistp`ed into
//! `camera_offset_z`.
//!
//! Caller: `physics::update_camera`.

use crate::effects::cell_x87::CellX87;
use crate::math::x87_float::X;
use crate::math::x87_float::to_int;
use crate::physics::camera_view::{record, share_done};
use crate::physics::step_families::CameraInputs;
use crate::race::owner_shared::RaceShared;

/// The exact version, for `physics::update_camera`.
pub fn ease_view_blend(
    c: &CameraInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    let esi = record(c.view_records, c.car)?;
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
    let t1 = shared.tables().view_share.ld();
    let t2 = t0 - t1;
    shared.tables().view_share.st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[camera_zoom] = (([view_shift_b] - [view_shift_a]) * [view_share]) + [view_shift_a]`
/// `int [camera_offset_z] = -(((([view_shift_d] - [view_shift_c]) * [view_share]) + [view_shift_c]) * [k65536])`
fn x87_2(c: &CameraInputs<'_>) {
    let t0 = c.camera.view_shift_b.ld();
    let t1 = c.camera.view_shift_a.ld();
    let t2 = t0 - t1;
    let t3 = c.race_tables.view_share.ld();
    let t4 = t2 * t3;
    let t5 = c.camera.view_shift_a.ld();
    let t6 = t4 + t5;
    c.car.camera_zoom.st(t6);
    let t7 = c.camera.view_shift_d.ld();
    let t8 = c.camera.view_shift_c.ld();
    let t9 = t7 - t8;
    let t10 = c.race_tables.view_share.ld();
    let t11 = t9 * t10;
    let t12 = c.camera.view_shift_c.ld();
    let t13 = t11 + t12;
    let t14 = c.numbers.k65536.ld();
    let t15 = t13 * t14;
    let t16 = -t15;
    c.camera.camera_offset_z.set(to_int(t16) as u32);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "ease_view_blend line 27",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x205260, In::F)],
                    writes: &[(0x205260, Out::F)],
                }],
                ulps: 0,
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
        sparse("physics.ease_view_blend", &blocks(), 1200, 100);
    }
}
