//! `physics::keep_split_time` (0x436B3D): the work car's time (`g_car` +0x88,
//! `unk_71A060`) kept in `dword_601C68[k]`, k = +0x90 (`unk_71A068`) less one, held to 0
//! ..= `g_splitColourCount` (both by the sign of the difference). A reading: a split
//! time at a checkpoint.
//!
//! In:  nothing
//! Out: eax the time, ebx k; the rest kept; no flags read (the caller `popa`s)
//!
//! Caller: `sub_436833`. Refuses (before anything) a k past the table's 0x40 entries (the
//! original writes past it; race init clears 0x40).

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::race::owner_physics::Physics;
use rc2k_core::physics::splits::kept_split_slot;

/// The Rust version. Converted: `rc2k_core::physics::splits::kept_split_slot` (below 0x40).
pub fn keep_split_time(physics: &Physics) -> Result<(), Refused> {
    let k = kept_split_slot(
        CAR.splits_passed.get(),
        physics.split_tables.split_colour_count.get(),
    )?;
    physics.split_tables.kept_times[k as usize].set(CAR.stage_time.get());
    Ok(())
}
