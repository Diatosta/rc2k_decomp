//! `physics::ease_view_angles` (0x43846D): with the mode's record's change time (+0x3C)
//! not 0: the share done (`physics/camera_view.rs`) times the record's three angle
//! changes (+0x40 .. +0x48) x `k65536` (`fistp`) << 16 added to `camera_offset_turns`,
//! `Car.camera_offset_turns[1]`, `Car.camera_offset_turns[2]`.
//!
//! Caller: `physics::update_camera`.

use crate::effects::cell_x87::CellX87;
use crate::math::x87_float::ld;
use crate::math::x87_float::to_int;
use crate::physics::camera_view::{record, share_done};
use crate::race::groups_store::ViewAndRecords;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::race::tables_store::RaceTables;

/// The exact version, for `physics::update_camera`.
pub fn ease_view_angles(
    numbers: &Numbers,
    shared: &RaceShared,
    view_records: &ViewAndRecords,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    let esi = record(view_records, shared.car())?;
    if esi.change_time.get() == 0 {
        return Ok(());
    }
    share_done(shared, esi, scene);
    for (to, rate) in shared.car().camera_offset_turns.iter().zip(&esi.turn_rates) {
        x87_1(numbers, shared.tables(), rate.get(), scene);
        to.set(
            to.get()
                .wrapping_add(scene.drawing.scene.get().scratch.get() << 16),
        );
    }
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [scratch] = ([s + 0] * [view_share]) * [k65536]` (`[s + 0]` a
/// record's turn rate, passed by value)
fn x87_1(numbers: &Numbers, tables: &RaceTables, rate: u32, scene: &crate::scene::owner::Scene) {
    let t0 = ld(rate);
    let t1 = tables.view_share.ld();
    let t2 = t0 * t1;
    let t3 = numbers.k65536.ld();
    let t4 = t2 * t3;
    scene.drawing.scene.get().scratch.set(to_int(t4) as u32);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "ease_view_angles line 26",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F), (0x205260, In::F)],
                    writes: &[(0x22400C, Out::I)],
                },
            ],
            ulps: 0,
            plain: |a| {
                x87_1(
                    &race().course.numbers,
                    race().shared.tables(),
                    crate::scene::x87_mem::Base::dword(a[0], 0),
                    crate::scene::owner::scene(),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.ease_view_angles", &blocks(), 1200, 100);
    }
}
