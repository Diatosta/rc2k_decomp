//! `saves::free_best_times` (0x47C627): gives back the memory of the best times' tables.
//!
//! In:  nothing
//! Out: every register preserved
//! `memory::free_list` on `g_bestTimesMemory` (0x7475D0), and `g_bestTimesAllocated`
//! (0x747610) = 0. The tables' addresses stay in their variables.
//! One caller, the main loop, after `saves::save_game_data`.
//! Refuses what `free_list` refuses.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{game_addr, rva};
use crate::memory::MemoryList;
use crate::memory::free_list::free_list;
use crate::saves::allocate_best_times::VARS_RVA as ALLOCATED_RVA;

// ---- game data ----

const TABLES_LIST_RVA: usize = rva(0x7475D0); // g_bestTimesMemory

// ---- Rust version ----

pub unsafe fn free_best_times(list: *mut MemoryList, allocated: *mut u32) -> Result<(), BadInput> {
    unsafe {
        free_list(list)?;
        allocated.write_unaligned(0);
        Ok(())
    }
}

/// The game's best-time tables freed.
pub fn free_in_game() -> Result<(), BadInput> {
    let list = game_addr(TABLES_LIST_RVA) as *mut MemoryList;
    let allocated = game_addr(ALLOCATED_RVA) as *mut u32;
    unsafe { free_best_times(list, allocated) }
}
