//! `saves::default_stage_times` (0x47C7FE): made-up best times for the stages of the six record tables
//! of `off_73555A`.
//!
//! In:  ebp (passed on as the stage to `default_time_entry`, which only uses it while `dword_755A0C` is 1)
//! Out: every register preserved
//! For each of the six tables and each of its records 1 .. 6: `seconds` (0x7559AC) = the
//! record's +0x18 + 24,000, and the ten entries the record's +0x3C picks in the table at
//! `dword_7556E0` get that and 0x8B more each, and a driver (`saves/default_times.rs`).
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses a missing table and a record that picks entries past the table's 36 stages.
//! Converted: `rc2k_core::saves::defaults::stage_times` (a refusal now leaves nothing written).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, rva};
use crate::saves::default_times::*;
use rc2k_core::saves::defaults::stage_times;

// ---- game data ----

const TABLE_RVA: usize = rva(0x7556E0); // dword_7556E0
const RECORD_TABLES_RVA: usize = rva(0x73555A); // off_73555A
const TABLES: usize = 6;
/// The stages the table has room for.
const ROOM: u32 = 36;

// ---- Rust version ----

pub unsafe fn default_stage_times(ebp: u32, log: Log) -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::defaults::stage_times`.
    unsafe {
        let table: u32 = game_read(TABLE_RVA);
        if table == 0 {
            return Err(BadInput("no table for the times").into());
        }
        let mut tables = Vec::new();
        for n in 0..TABLES {
            let records: u32 = game_read(RECORD_TABLES_RVA + n * 4);
            if records == 0 || records == u32::MAX {
                return Err(BadInput("a record table is missing").into());
            }
            tables.push(std::array::from_fn(|stage| {
                read_record(records + (stage as u32 + 1) * RECORD_LEN)
            }));
        }
        let filled = stage_times(
            &mut GameSource { log },
            &tables,
            Some(ebp),
            with_checkpoints(),
        )?;
        apply(&filled, &[(table, (ROOM * STAGE_LEN) as usize)]);
        Ok(())
    }
}
