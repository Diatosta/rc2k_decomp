//! `saves::default_championship_times` (0x47C973): made-up best times for the
//! championship's stages and rallies.
//!
//! In:  ebp (passed on as the stage to `default_time_entry`, which only uses it while
//!      `dword_755A0C` is 1)
//! Out: every register preserved
//! For each stage of the championship's record table (its records with tag 0, counted
//! into `seconds`, 0x7559AC): the ten entries of the stage in the table at
//! `dword_7556D8` get the record's +0x24 + 24,000 and 0x9F more each, and a driver
//! (`saves/default_times.rs`). Then, going through the records once more: a stage adds
//! its +0x18 + 0xB6 to `dword_7559B0`; a record with tag 3 or 4 (a rally's end) gives the
//! twenty entries of the next of the six tables of `off_742C98` that sum and 0xB2 more
//! each at +0x18, and a driver (`sub_47CB0A`), and the sum starts again from 0.
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses missing tables, a record table without stages or without its end, and more
//! rallies than the six tables. Converted: `rc2k_core::saves::defaults::championship_times`
//! (a refusal now leaves nothing written).
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, rva};
use crate::saves::allocate_best_times::count_stages;
use crate::saves::default_times::*;
use rc2k_core::saves::defaults::{Championship, RALLIES, RALLY_TABLE_LEN, championship_times};

// ---- game data ----

const RECORD_TABLE_RVA: usize = rva(0x735576); // g_recordTable
const TABLE_RVA: usize = rva(0x7556D8); // dword_7556D8
const RALLY_TABLES_RVA: usize = rva(0x742C98); // off_742C98

// ---- Rust version ----

pub unsafe fn default_championship_times(ebp: u32, log: Log) -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::defaults::championship_times`.
    unsafe {
        let records_at = game_addr(RECORD_TABLE_RVA) as u32;
        let stages = count_stages(records_at as *const u8)?;
        let count = records(records_at)?;
        let table: u32 = game_read(TABLE_RVA);
        if stages == 0 || table == 0 {
            return Err(BadInput("no stages, or no table for their times").into());
        }
        let records: Vec<_> = (0..count as u32)
            .map(|n| read_record(records_at + n * RECORD_LEN))
            .collect();
        let mut tables = vec![(table, (stages * STAGE_LEN) as usize)];
        let mut present = [false; RALLIES];
        for (rally, there) in present.iter_mut().enumerate() {
            let entries: u32 = game_read(RALLY_TABLES_RVA + rally * 4);
            *there = entries != 0;
            tables.push((entries, RALLY_TABLE_LEN));
        }
        let filled = championship_times(
            &mut GameSource { log },
            &records,
            Championship::Times { stage: ebp },
            present,
            with_checkpoints(),
        )?;
        apply(&filled, &tables);
        Ok(())
    }
}
