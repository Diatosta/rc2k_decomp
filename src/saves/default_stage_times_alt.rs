//! `saves::default_stage_times_alt` (0x47C86F): made-up best times for the stages of the six record tables
//! of `off_73555A`.
//!
//! For each of the six tables and each of its records 1 .. 6: `seconds` (0x7559AC) = the
//! record's +0x18 + 24,000, and the ten entries the record's +0x3C picks in the table at
//! `dword_7556E4` get that and 0x8B more each, and a driver (`saves/default_times.rs`), with the +0x3C as the stage for the checkpoints. How this function and `saves::default_stage_times` differ in use is not told by the code (one caller each; left open).
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses a missing table and a record that picks entries past the table's 36 stages.
//! Converted: `rc2k_core::saves::defaults::stage_times` (a refusal now leaves nothing written).

use crate::errors::{BadInput, Refused};
use crate::saves::default_times::*;
use crate::system::owner::system;
use rc2k_core::saves::defaults::stage_times;

// ---- game data ----

const TABLES: usize = 6;

/// `ebp` is not used: the stage comes from the record.
pub fn default_stage_times_alt(_ebp: u32, log: Log) -> Result<(), Refused> {
    let screens = &crate::menu::owner::menus().screens;
    // Converted: `rc2k_core::saves::defaults::stage_times`.
    {
        let table = &crate::saves::best_times::STAGE_TIMES_ALT;
        if !table.is_taken() {
            return Err(BadInput("no table for the times").into());
        }
        let mut tables = Vec::new();
        for n in 0..TABLES {
            let records = system().records.stage_tables.level_table(n as u32)?;
            let six = (0..6u32)
                .map(|stage| read_record(records.row(stage + 1)?))
                .collect::<Result<Vec<_>, _>>()?;
            tables.push(
                six.try_into()
                    .map_err(|_| BadInput("a table of other than six records"))?,
            );
        }
        let filled = stage_times(
            &mut GameSource { log },
            &tables,
            None,
            with_checkpoints(screens),
        )?;
        apply(screens, &filled, &[Target::Stages(table)])?;
        Ok(())
    }
}
