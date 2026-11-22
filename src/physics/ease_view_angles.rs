//! With the mode record's change time not 0: the share done (`physics/camera_view.rs`) times
//! the record's three angle changes, times `k65536`, rounded and shifted left 16, is added to
//! `camera_offset_turns`.

use crate::math::cell_x87::CellX87;
use crate::math::x87_float::ld;
use crate::math::x87_float::to_int;
use crate::memory::session::DrawArc;
use crate::physics::camera_view::{record, share_done};
use crate::race::groups_store::ViewAndRecords;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::race::tables_store::RaceTables;

/// The exact version, for `physics::update_camera`.
pub fn ease_view_angles(
    numbers: &Numbers,
    shared: &RaceShared,
    view_records: &DrawArc<ViewAndRecords>,
) -> Result<(), crate::errors::BadInput> {
    let view_record = record(view_records, shared.car())?;
    if view_record.change_time == 0 {
        return Ok(());
    }
    share_done(shared, &view_record);
    let scratch = crate::memory::session::PlainDword::new(0);
    for (k, rate) in view_record.turn_rates.iter().enumerate().take(3) {
        x87_1(numbers, shared.tables(), *rate, &scratch);
        let turns = &shared.car().camera_offset_turns;
        turns.set_bits_at(k, turns.bits_at(k).wrapping_add(scratch.get() << 16));
    }
    Ok(())
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [scratch] = ([s + 0] * [view_share]) * [k65536]` (`[s + 0]` a record's turn rate, passed by
///  value)
fn x87_1(
    numbers: &Numbers,
    tables: &RaceTables,
    rate: u32,
    scratch: &crate::memory::session::PlainDword,
) {
    let t0 = ld(rate);
    let t1 = tables.view_share.ld();
    let t2 = t0 * t1;
    let t3 = numbers.k65536.ld();
    let t4 = t2 * t3;
    scratch.set(to_int(t4) as u32);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::own::race;
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
                    &crate::test_support::race_places::numbers(),
                    race().shared.tables(),
                    crate::math::x87_mem::Base::dword(a[0], 0),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.ease_view_angles", &blocks(), 1200, 100);
    }
}
