//! The camera's view steps under `physics::update_camera`: the mode record
//! (`view_records`[mode]`, the mode `camera_view` held to 0 ..= 0x33), the share of a
//! change done (`step_clock` - `camera_switch_clock` held to 0 ..= the record's +0x3C, over it,
//! into `view_share`), and the tests' frame of `physics::take_first_view`, `take_second_view`,
//! `ease_view_angles`, `ease_view_blend`, `camera_height_bias`, `ease_view_turn`,
//! `place_camera`. The converted ones run `share_done_plain`, `record_plain` and
//! `view_test_floats` (`rc2k_core::physics::camera_view`). What the variables are (the
//! camera's mode, place, turns, a view's figures) is a reading.

use crate::math::x87_float::int;
use crate::math::x87_float::st;
use crate::physics::car_store::Car;
use crate::race::owner_shared::RaceShared;
use crate::race::view_vars_store::{ViewRecord, ViewVars};

/// The record of the car's camera mode.
pub fn record(car: &Car) -> Result<&ViewRecord, crate::errors::BadInput> {
    let mode = (car.camera_view.get() as i32).clamp(0, 0x33);
    ViewVars::record_of(mode as usize)
}

/// The share done (the time held).
pub fn share_done(
    shared: &RaceShared,
    record: &ViewRecord,
    scene: &crate::scene::owner::Scene,
) -> u32 {
    let mut eax = shared
        .car
        .step_clock
        .get()
        .wrapping_sub(shared.car.camera_switch_clock.get());
    if (eax as i32) < 0 {
        eax = 0;
    }
    let change_time = record.change_time.get();
    if eax >= change_time {
        eax = change_time;
    }
    scene.drawing.scene.scratch.set(eax);
    x87_1(shared, change_time, scene);
    eax
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[view_share] = int [scratch] / int [s + 0x3C]` (`s + 0x3C` the record's
/// `change_time`, passed by value)
fn x87_1(shared: &RaceShared, change_time: u32, scene: &crate::scene::owner::Scene) {
    let t0 = int(scene.drawing.scene.scratch.bits() as i32);
    let t1 = int(change_time as i32);
    let t2 = t0 / t1;
    shared.tables.view_share.set_bits(st(t2));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc, s) "fild dword ptr [{rc} + 0x22400C]", "fidiv dword ptr [{s} + 0x3C]", "fstp dword ptr [{rc} + 0x205260]");

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
            reference: |a| unsafe { ref_0(a[0], a[1]) },
            plain: |a| {
                x87_1(
                    &race().shared,
                    crate::scene::x87_mem::Base::dword(a[1], 0x3C),
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
