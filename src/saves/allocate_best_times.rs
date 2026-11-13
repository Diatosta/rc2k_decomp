//! Takes the memory of the best times' tables, once.
//!
//! Nothing if the tables are already allocated; else the flag becomes 1 and nine tables are
//! taken on the memory list, each one's size stored in the pairs of `tables`: 0x1C20, 0xBB8,
//! stages * 0x618, 0x15E0, 0xDB60, 0x4C2C4, 0x3A984, 0xDB60, stages * 0x618 bytes. The sixth
//! and the seventh get -1 in their last dword. "Stages" is the number of records of the
//! championship's table whose tag is 0 (`saves::count_stages`). Six of the nine are what
//! `saves::load_best_times` reads `var\hst.dat` into.
//! Called by `saves::load_best_times` and `menu::set_up_front_end`. Refuses a record table
//! without its end, and what the allocator refuses.

use crate::errors::{BadInput, Refused};
use crate::records::stages_store::{RecordTable, StageTables};
use crate::saves::best_times_store::BestTimes;
use rc2k_core::saves::tables::TABLES;

/// Where table `n`'s address is among the variables; its size follows.
pub fn count_stages(stages: &StageTables, table: RecordTable) -> Result<u32, BadInput> {
    let fault = crate::menu::event_flow::Fault::default();
    let answer = rc2k_core::saves::tables::count_stages(|n| {
        fault.ok(table.row(stages, n).map(|r| r.tag(stages)))
    });
    fault.check()?;
    answer.map_err(BadInput::from)
}

pub fn sizes(stages: u32) -> [u32; TABLES] {
    rc2k_core::saves::tables::sizes(stages)
}

/// Once, the nine tables taken from `list` (typed, `saves::best_times`, counted on it;
/// `saves().best_times.tables` keeps each one's size), the marked ones ending in -1 (each
/// typed table's own end).
pub fn allocate_best_times(
    best_times: &BestTimes,
    stages: &StageTables,
    records: RecordTable,
) -> Result<(), Refused> {
    let list = best_times.list();
    if best_times.allocated.get() == 1 {
        return Ok(());
    }
    let sizes = sizes(count_stages(stages, records)?);
    best_times.allocated.set(1);
    for (n, size) in sizes.into_iter().enumerate() {
        let place = &best_times.tables[n];
        place.size.set(size);
        crate::saves::best_times::typed(n).take(list, size)?;
    }
    Ok(())
}
