//! Keeps the work car's `stage_time` in `kept_times[k]`, k = `splits_passed` less one, held to 0
//! ..= `split_colour_count`. A reading: a split time at a checkpoint.
//!
//! Refuses (before anything) a k past the table's 0x40 entries.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::split_tables_store::SplitTables;
use rc2k_core::physics::splits::kept_split_slot;

/// Core: `rc2k_core::physics::splits::kept_split_slot` (below 0x40).
pub(super) fn keep_split_time(car: &Car, split_tables: &SplitTables) -> Result<(), Refused> {
    let k = kept_split_slot(
        car.splits_passed.get(),
        split_tables.split_colour_count.get(),
    )?;
    split_tables.kept_times[k as usize].set(car.stage_time.get());
    Ok(())
}
