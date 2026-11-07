//! The camera's view steps under `sub_437A53` (stream B, 2026-10-18): the mode record
//! (`off_606320[mode]`, the mode `unk_71A65C` held to 0 ..= 0x33), the share of a
//! change done (`unk_719FE0` - `unk_71A668` held to 0 ..= the record's +0x3C, over it,
//! into `flt_605260`), and the tests' frame of `physics::take_view_a`, `take_view_b`,
//! `ease_view_angles`, `ease_view_blend`, `camera_height_bias`, `ease_view_turn`,
//! `place_camera`. The converted ones run `share_done_plain`, `record_plain` and
//! `view_test_floats` (`rc2k_core::physics::camera_view`). What the variables are (the
//! camera's mode, place, turns, a view's figures) is a reading.

use crate::math::x87_float::int;
use crate::math::x87_float::st;
use crate::race::owner_shared::Shared;
use crate::race::view_vars_store::{ViewRecord, ViewVars};

/// esi as the original leaves it: the mode's record.
pub fn record(shared: &Shared) -> Result<&ViewRecord, crate::errors::BadInput> {
    let mode = (shared.car.camera_view.get() as i32).clamp(0, 0x33);
    ViewVars::record_of(mode as usize)
}

/// The share done (0x438498 .. 0x4384BD): eax as left (the time held).
pub fn share_done(shared: &Shared, record: &ViewRecord, scene: &crate::scene::owner::Scene) -> u32 {
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

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x205260] = int [rc + 0x22400C] / int [s + 0x3C]` (`s + 0x3C` the record's
/// `change_time`, passed by value)
fn x87_1(shared: &Shared, change_time: u32, scene: &crate::scene::owner::Scene) {
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
