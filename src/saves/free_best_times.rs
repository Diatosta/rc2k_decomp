//! Gives back the memory of the best times' tables.
//!
//! `memory::free_list` on the best times' list, and the allocated flag = 0. The tables'
//! addresses stay in their variables (the typed tables, `saves::best_times`, go with the list).
//! One caller, the main loop, after `saves::save_game_data`. Refuses what `free_list` refuses.

use crate::errors::BadInput;
use crate::memory::free_list::free_list;
use crate::saves::best_times_store::BestTimes;
use crate::saves::owner::Saves;

pub fn free_best_times(best_times: &BestTimes) -> Result<(), BadInput> {
    free_list(best_times.list())?;
    crate::saves::best_times::give_back_all();
    best_times.allocated.set(0);
    Ok(())
}

pub fn free_in_game(saves: &Saves) -> Result<(), BadInput> {
    free_best_times(&saves.best_times)
}
