//! `physics::camera_height_bias` (0x43852D): with the mode's record's +0x54 not 0:
//! `camera_distance` / `camera_bias_limit` held to 0 ..= 1 (1 less it for +0x54 below 0), x
//! `k65536` x `camera_bias_scale` (`fistp`) into `camera_height_bias` and added to the camera's
//! height `Car.camera_position[1]`.
//!
//! Caller: `physics::update_camera`.

use crate::effects::cell_x87::CellX87;
use crate::math::x87_float::X;
use crate::math::x87_float::to_int;
use crate::physics::camera_view::record;
use crate::physics::car_store::Car;
use crate::physics::flow::{ZERO_BITS as ZERO, hold_high, hold_low_bits};
use crate::race::groups_store::ViewAndRecords;
use crate::race::numbers_store::Numbers;

/// The exact version, for `physics::update_camera`.
pub fn camera_height_bias(
    numbers: &Numbers,
    view_records: &ViewAndRecords,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    let drawing = &scene.drawing;
    let esi = record(view_records, car)?;
    if esi.height_bias.get() == 0 {
        return Ok(());
    }
    x87_1(view_records, scene);
    hold_low_bits(&drawing.scene.get().scratch, ZERO);
    hold_high(&drawing.scene.get().scratch, &numbers.one);
    if (esi.height_bias.get() as i32) < 0 {
        x87_2(scene);
    }
    x87_3(numbers, view_records, scene);
    let eax = drawing.scene.get().scratch.get();
    view_records.view_vars.camera_height_bias.set(eax);
    car.camera_position[1].set(car.camera_position[1].get().wrapping_add(eax));
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[scratch] = [camera_distance] / [camera_bias_limit]`
fn x87_1(view_records: &ViewAndRecords, scene: &crate::scene::owner::Scene) {
    let t0 = view_records.view_vars.camera_distance.ld();
    let t1 = view_records.view_vars.camera_bias_limit.ld();
    let t2 = t0 / t1;
    scene.drawing.scene.get().scratch.st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[scratch] = 1 - [scratch]`
fn x87_2(scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = X(1.0);
    let t1 = drawing.scene.get().scratch.ld();
    let t2 = t0 - t1;
    drawing.scene.get().scratch.st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [scratch] = ([scratch] * [k65536]) * [camera_bias_scale]`
fn x87_3(numbers: &Numbers, view_and_records: &ViewAndRecords, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = drawing.scene.get().scratch.ld();
    let t1 = numbers.k65536.ld();
    let t2 = t0 * t1;
    let t3 = view_and_records.view_vars.camera_bias_scale.ld();
    let t4 = t2 * t3;
    drawing.scene.get().scratch.set(to_int(t4) as u32);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "camera_height_bias line 24",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x206290, In::F), (0x20631C, In::F)],
                    writes: &[(0x22400C, Out::F)],
                }],
                ulps: 0,
                plain: |_| {
                    x87_1(
                        crate::race::owner::race().frame.view_and_records.get(),
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
                plain: |_| x87_2(crate::scene::owner::scene()),
            },
            Block {
                name: "camera_height_bias line 40",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F), (0x20628C, In::F), (0x22400C, In::F)],
                    writes: &[(0x22400C, Out::I)],
                }],
                ulps: 0,
                plain: |_| {
                    x87_3(
                        crate::race::owner::race().course.numbers.get(),
                        crate::race::owner::race().frame.view_and_records.get(),
                        crate::scene::owner::scene(),
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.camera_height_bias", &blocks(), 1200, 100);
    }
}
