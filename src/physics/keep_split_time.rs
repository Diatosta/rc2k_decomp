//! `physics::keep_split_time` (0x436B3D): the work car's time (`g_car` +0x88,
//! `unk_71A060`) kept in `dword_601C68[k]`, k = +0x90 (`unk_71A068`) less one, held to 0
//! ..= `g_splitColourCount` (both by the sign of the difference). A reading: a split
//! time at a checkpoint.
//!
//! Returns: the time, k
//!
//! Caller: `physics::take_split_counted`. Refuses (before anything) a k past the table's 0x40
//! entries (the original writes past it; race init clears 0x40).

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::physics::splits::kept_split_slot;

/// The Rust version. Converted: `rc2k_core::physics::splits::kept_split_slot` (below 0x40).
pub fn keep_split_time(race: &Race) -> Result<(), Refused> {
    let k = kept_split_slot(
        race.shared.car.splits_passed.get(),
        race.physics.split_tables.split_colour_count.get(),
    )?;
    race.physics.split_tables.kept_times[k as usize].set(race.shared.car.stage_time.get());
    Ok(())
}
