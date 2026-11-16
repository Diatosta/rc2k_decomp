//! Made-up best times for the championship's stages and rallies: the ten entries of each stage
//! get the record's fourth class time + 24,000 and 0x9F more each, and a driver
//! (`saves/default_times.rs`); a rally's end (tag 3 or 4) gives the twenty entries of the next of
//! the six rally tables the running sum of the stages' first class times + 0xB6 and 0xB2 more
//! each, and a driver. With the checkpoint times asked for, those are made too. One caller,
//! `menu::set_up_front_end`, when `var\hst.dat` was not loaded. Refuses missing tables, a record
//! table without stages or without its end, and more rallies than the six tables; a refusal
//! leaves nothing written.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::saves::allocate_best_times::count_stages;
use crate::saves::default_times::*;
use crate::saves::owner::Saves;
use crate::system::owner::System;
use rc2k_core::saves::defaults::{Championship, RALLIES, championship_times};

pub fn default_championship_times(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    race: &Race,
    system: &System,
    saves: &Saves,
    ebp: u32,
    log: Log,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    {
        let tables = &system.records().stage_tables;
        let records_at = tables.championship();
        let stages = count_stages(tables, records_at)?;
        let count = records(tables, records_at)?;
        let table = &saves.best_tables.stage_times;
        let (true, true) = (stages != 0, table.is_taken()) else {
            return Err(BadInput("no stages, or no table for their times").into());
        };
        let records: Vec<_> = (0..count as u32)
            .map(|n| read_record(tables, records_at.row(tables, n)?))
            .collect::<Result<_, _>>()?;
        let mut tables = vec![Target::Stages(table)];
        // Every rally has its table.
        let present = [true; RALLIES];
        for rally in 0..RALLIES {
            tables.push(Target::Rally(saves.rally_times.table(rally)?));
        }
        let filled = championship_times(
            &mut GameSource {
                log,
                so,
                menus,
                race,
            },
            &records,
            Championship::Times { stage: ebp },
            present,
            false,
        )?;
        apply(screens, race, &filled, &tables)?;
        Ok(())
    }
}
