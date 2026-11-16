//! Made-up best times with checkpoint times for the championship's stages: for each stage of the
//! championship's record table the ten entries of the stage in the stage times table get the
//! record's first class time + 24,000 and 0x9F more each, and a driver, with the checkpoint times
//! guessed (some 9,000 pieces go to the game's log). The running sum of the second pass of
//! `default_championship_times` is added up and cleared but not used. Refuses a missing table and a
//! record table without stages or without its end; a refusal leaves nothing written.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::saves::allocate_best_times::count_stages;
use crate::saves::best_times::BestTables;
use crate::saves::default_times::*;
use crate::system::owner::System;
use rc2k_core::saves::defaults::{Championship, RALLIES, championship_times};

pub fn default_checkpoint_times(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    race: &Race,
    best_tables: &BestTables,
    system: &System,
    log: Log,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    {
        let tables = &system.records().stage_tables;
        let records_at = tables.championship();
        let stages = count_stages(tables, records_at)?;
        let count = records(tables, records_at)?;
        let table = &best_tables.time_trial_times;
        let (true, true) = (stages != 0, table.is_taken()) else {
            return Err(BadInput("no stages, or no table for their times").into());
        };
        let records: Vec<_> = (0..count as u32)
            .map(|n| read_record(tables, records_at.row(tables, n)?))
            .collect::<Result<_, _>>()?;
        let tables = [Target::Stages(table)];
        let filled = championship_times(
            &mut GameSource {
                log,
                so,
                menus,
                race,
            },
            &records,
            Championship::Checkpoints,
            [false; RALLIES],
            true,
        )?;
        apply(screens, race, &filled, &tables)?;
        Ok(())
    }
}
