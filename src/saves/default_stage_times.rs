//! `saves::default_stage_times` (0x47C7FE): made-up best times for the stages of the six record tables
//! of the stage map.
//!
//! For each of the six tables and each of its records 1 .. 6: `seconds` = the
//! record's +0x18 + 24,000, and the ten entries the record's +0x3C picks in the stage times
//! table get that and 0x8B more each, and a driver (`saves/default_times.rs`).
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses a missing table and a record that picks entries past the table's 36 stages.
//! Converted: `rc2k_core::saves::defaults::stage_times` (a refusal now leaves nothing written).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::saves::default_times::*;
use crate::system::owner::System;
use rc2k_core::saves::defaults::stage_times;

// ---- game data ----

const TABLES: usize = 6;

pub fn default_stage_times(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    race: &Race,
    system: &System,
    ebp: u32,
    log: Log,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    // Converted: `rc2k_core::saves::defaults::stage_times`.
    {
        let table = &crate::saves::best_times::COURSE_TIMES;
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
            Some(ebp),
            with_checkpoints(screens),
        )?;
        apply(screens, race, &filled, &[Target::Stages(table)])?;
        Ok(())
    }
}
