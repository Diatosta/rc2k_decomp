//! `saves::free_best_times` (0x47C627): gives back the memory of the best times' tables.
//!
//! In:  nothing
//! Out: every register preserved
//! `memory::free_list` on `g_bestTimesMemory` (0x7475D0), and `g_bestTimesAllocated` (0x747610) =
//! 0. The tables' addresses stay in their variables (the typed tables, `saves::best_times`,
//! go with the list). One caller, the main loop, after
//! `saves::save_game_data`. Refuses what `free_list` refuses. Rust callers run its `entry`
//! (`routes.rs`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::memory::free_list::free_list;
use crate::saves::best_times_store::BestTimes;
use crate::saves::owner::Saves;

// ---- game data ----

// ---- Rust version ----

pub fn free_best_times(best_times: &BestTimes) -> Result<(), BadInput> {
    free_list(best_times.list())?;
    crate::saves::best_times::give_back_all();
    best_times.allocated.set(0);
    Ok(())
}

pub fn free_in_game(saves: &Saves) -> Result<(), BadInput> {
    free_best_times(&saves.best_times)
}
