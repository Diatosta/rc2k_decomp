//! Made-up best times for the stages of the six record tables of the stage map.
//!
//! For each of the six tables and each of its records 1 .. 6: `seconds` = the record's first
//! class time + 24,000, and the ten entries the record's event slot picks in the alternate
//! stage times table get that and 0x8B more each, and a driver (`saves/default_times.rs`),
//! with the same slot as the stage for the checkpoints. How it and `saves::default_stage_times`
//! differ in use is open (one caller each).
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded. Refuses a missing
//! table and a record that picks entries past the table's 36 stages. A refusal leaves nothing
//! written.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::saves::best_times::BestTables;
use crate::saves::default_times::*;
use crate::system::owner::System;
use rc2k_core::saves::defaults::stage_times;

const TABLES: usize = 6;

/// The first argument is not used: the stage comes from the record.
pub fn default_stage_times_alt(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    race: &Race,
    best_tables: &BestTables,
    system: &System,
    _ebp: u32,
    log: Log,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    {
        let table = &best_tables.stage_times_alt;
        if !table.is_taken() {
            return Err(BadInput("no table for the times").into());
        }
        let mut tables = Vec::new();
        for n in 0..TABLES {
            let stage_tables = &system.records().stage_tables;
            let records = stage_tables.level_table(n as u32)?;
            let six = (0..6u32)
                .map(|stage| read_record(stage_tables, records.row(stage_tables, stage + 1)?))
                .collect::<Result<Vec<_>, _>>()?;
            tables.push(
                six.try_into()
                    .map_err(|_| BadInput("a table of other than six records"))?,
            );
        }
        let filled = stage_times(
            &mut GameSource {
                log,
                so,
                menus,
                race,
            },
            &tables,
            None,
            false,
        )?;
        apply(screens, race, &filled, &[Target::Stages(table)])?;
        Ok(())
    }
}
