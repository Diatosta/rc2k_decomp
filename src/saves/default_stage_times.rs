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
use crate::saves::default_times::*;
use rc2k_core::saves::defaults::stage_times;

// ---- game data ----

const TABLES: usize = 6;

// ---- Rust version ----

pub fn default_stage_times(ebp: u32, log: Log) -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::defaults::stage_times`.
    {
        let table = &crate::saves::best_times::COURSE_TIMES;
        if !table.is_taken() {
            return Err(BadInput("no table for the times").into());
        }
        let mut tables = Vec::new();
        for n in 0..TABLES {
            let records = crate::records::stages_store::STAGE_TABLES.level_table(n as u32)?;
            let six = (0..6u32)
                .map(|stage| read_record(records + (stage + 1) * RECORD_LEN))
                .collect::<Result<Vec<_>, _>>()?;
            tables.push(
                six.try_into()
                    .map_err(|_| BadInput("a table of other than six records"))?,
            );
        }
        let filled = stage_times(
            &mut GameSource { log },
            &tables,
            Some(ebp),
            with_checkpoints(),
        )?;
        apply(&filled, &[Target::Stages(table)])?;
        Ok(())
    }
}
