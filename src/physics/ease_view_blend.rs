//! With the mode record's blend and change time not 0: the share done
//! (`physics/camera_view.rs`; 1 less it for a blend below 0) between `view_shift_a` and
//! `view_shift_b` goes into `camera_zoom`, and between `view_shift_c` and `view_shift_d`, times
//! `k65536`, negated and rounded, into `camera_offset_z`.

use crate::math::cell_x87::CellX87;
use crate::math::cell_x87::PlainX87;
use crate::math::x87_float::to_int;
use crate::math::x87_float::{X, ld};
use crate::physics::camera_view::{record, share_done};
use crate::physics::car_store::Car;
use crate::physics::step_families::CameraInputs;
use crate::race::owner_shared::RaceShared;

/// The exact version, for `physics::update_camera`.
pub fn ease_view_blend(c: &CameraInputs<'_>, car: &mut Car) -> Result<(), crate::errors::BadInput> {
    let view_record = record(c.view_records, car)?;
    if view_record.blend == 0 || view_record.change_time == 0 {
        return Ok(());
    }
    share_done(c.shared, car, &view_record);
    if (view_record.blend as i32) < 0 {
        x87_1(c.shared);
    }
    x87_2(c, car);
    Ok(())
}

// ---- the x87 arithmetic ----

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
/// `int [camera_offset_z] = -(((([view_shift_d] - [view_shift_c]) * [view_share]) + [view_shift_c])
///  * [k65536])`
fn x87_2(c: &CameraInputs<'_>, car: &mut Car) {
    let camera = c.camera.borrow();
    let t0 = ld(camera.view_shift_b);
    let t1 = ld(camera.view_shift_a);
    let t2 = t0 - t1;
    let t3 = c.race_tables.view_share.ld();
    let t4 = t2 * t3;
    let t5 = ld(camera.view_shift_a);
    let t6 = t4 + t5;
    car.camera_zoom.st(t6);
    let t7 = ld(camera.view_shift_d);
    let t8 = ld(camera.view_shift_c);
    let t9 = t7 - t8;
    let t10 = c.race_tables.view_share.ld();
    let t11 = t9 * t10;
    let t12 = ld(camera.view_shift_c);
    let t13 = t11 + t12;
    let t14 = c.numbers.k65536.ld();
    let t15 = t13 * t14;
    let t16 = -t15;
    drop(camera);
    c.camera.borrow_mut().camera_offset_z = to_int(t16) as u32;
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::own::race;
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
                    let numbers = crate::test_support::race_places::numbers();
                    let inputs = crate::physics::step_inputs::StepInputs::of(
                        crate::test_support::own::race(),
                    )
                    .camera();
                    x87_2(
                        &CameraInputs {
                            numbers: &numbers,
                            ..inputs
                        },
                        &mut crate::test_support::own::race().shared.car_mut(),
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
