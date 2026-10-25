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
use crate::physics::split_tables_store::SPLIT_TABLES;
use rc2k_core::physics::splits::kept_split_slot;

/// Where the variables are: the time, the checks, the count, the table.
#[derive(Clone, Copy)]
pub struct SplitPlaces {
    pub time: u32,
    pub checks: u32,
    pub count: u32,
    pub table: u32,
}

impl SplitPlaces {
    pub fn game() -> Self {
        Self {
            time: (CAR.stage_time.addr() as u32),
            checks: (CAR.splits_passed.addr() as u32),
            count: (SPLIT_TABLES.split_colour_count.addr() as u32),
            table: (SPLIT_TABLES.kept_times[0].addr() as u32),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version on the variables at `p`.
// Converted: `rc2k_core::physics::splits::kept_split_slot`.
pub unsafe fn keep_split_time_at(p: &SplitPlaces) -> Result<(), Refused> {
    unsafe {
        let time = at(p.time);
        let k = kept_split_slot(at(p.checks), at(p.count))?;
        ((p.table + 4 * k) as *mut u32).write_unaligned(time);
        Ok(())
    }
}

/// The Rust version.
pub unsafe fn keep_split_time() -> Result<(), Refused> {
    unsafe { keep_split_time_at(&SplitPlaces::game()) }
}
