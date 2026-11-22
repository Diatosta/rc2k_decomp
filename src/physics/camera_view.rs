//! The camera's view steps under `physics::update_camera`: the mode record (`view_records`, the
//! mode `camera_view` held to 0 ..= 0x33), the share of a change done (`step_clock` less
//! `camera_switch_clock`, held to 0 ..= the record's length and divided by it into `view_share`),
//! and the tests' frame of `physics::take_first_view`, `take_second_view`, `ease_view_angles`,
//! `ease_view_blend`, `camera_height_bias`, `ease_view_turn` and `place_camera`. The converted
//! steps `share_done` and `record` run on plain floats (`rc2k_core::physics::camera_view`). The
//! names of the variables (mode, place, turns, a view's figures) are a reading.

use crate::math::cell_x87::CellX87;
use crate::math::x87_float::int;
use crate::memory::session::DrawArc;
use crate::physics::car_store::Car;
use crate::race::groups_store::ViewAndRecords;
use crate::race::owner_shared::RaceShared;
use crate::race::view_vars_store::{ViewRecord, ViewVars};

/// The record of the car's camera mode.
pub fn record(
    view_records: &DrawArc<ViewAndRecords>,
    car: &Car,
) -> Result<ViewRecord, crate::errors::BadInput> {
    let mode = rc2k_core::physics::camera_view::mode_index(car.camera_view.get());
    ViewVars::record_of(&view_records.borrow(), mode as usize).copied()
}

/// The share done (the time held).
pub fn share_done(shared: &RaceShared, record: &ViewRecord) -> u32 {
    let mut time_held = shared
        .car()
        .step_clock
        .get()
        .wrapping_sub(shared.car().camera_switch_clock.get());
    if (time_held as i32) < 0 {
        time_held = 0;
    }
    let change_time = record.change_time;
    if time_held >= change_time {
        time_held = change_time;
    }
    x87_1(shared, time_held, change_time);
    time_held
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[view_share] = int [done] / int [change_time]` (the share done and the record's `change_time`,
/// passed by value)
fn x87_1(shared: &RaceShared, done: u32, change_time: u32) {
    let t0 = int(done as i32);
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
                    crate::test_support::own::scene().draw_work().scratch,
                    crate::math::x87_mem::Base::dword(a[1], 0x3C),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.camera_view", &blocks(), 1200, 100);
    }
}
