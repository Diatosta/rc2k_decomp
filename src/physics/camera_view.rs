//! The camera's view steps under `physics::update_camera`: the mode record (`view_records`, the
//! mode `camera_view` held to 0 ..= 0x33), the share of a change done (`step_clock` less
//! `camera_switch_clock`, held to 0 ..= the record's length and divided by it into `view_share`),
//! and the tests' frame of `physics::take_first_view`, `take_second_view`, `ease_view_angles`,
//! `ease_view_blend`, `camera_height_bias`, `ease_view_turn` and `place_camera`. The converted
//! steps `share_done` and `record` run on plain floats (`rc2k_core::physics::camera_view`). The
//! names of the variables (mode, place, turns, a view's figures) are a reading.

use crate::effects::cell_x87::CellX87;
use crate::math::x87_float::int;
use crate::physics::car_store::Car;
use crate::race::groups_store::ViewAndRecords;
use crate::race::owner_shared::RaceShared;
use crate::race::view_vars_store::{ViewRecord, ViewVars};

/// The record of the car's camera mode.
pub fn record<'a>(
    view_records: &'a ViewAndRecords,
    car: &Car,
) -> Result<&'a ViewRecord, crate::errors::BadInput> {
    let mode = (car.camera_view.get() as i32).clamp(0, 0x33);
    ViewVars::record_of(view_records, mode as usize)
}

/// The share done (the time held).
pub fn share_done(
    shared: &RaceShared,
    record: &ViewRecord,
    scene: &crate::scene::owner::Scene,
) -> u32 {
    let mut eax = shared
        .car()
        .step_clock
        .get()
        .wrapping_sub(shared.car().camera_switch_clock.get());
    if (eax as i32) < 0 {
        eax = 0;
    }
    let change_time = record.change_time.get();
    if eax >= change_time {
        eax = change_time;
    }
    scene.drawing.scene.get().scratch.set(eax);
    x87_1(shared, change_time, scene);
    eax
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[view_share] = int [scratch] / int [change_time]` (the record's `change_time`, passed by value)
fn x87_1(shared: &RaceShared, change_time: u32, scene: &crate::scene::owner::Scene) {
    let t0 = int(scene.drawing.scene.get().scratch.get() as i32);
    let t1 = int(change_time as i32);
    let t2 = t0 / t1;
    shared.tables().view_share.st(t2);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::own::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "camera_view line 35",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x22400C, In::I)],
                    writes: &[(0x205260, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0x3C, In::I)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                x87_1(
                    &race().shared,
                    crate::scene::x87_mem::Base::dword(a[1], 0x3C),
                    crate::test_support::own::scene(),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.camera_view", &blocks(), 1200, 100);
    }
}
