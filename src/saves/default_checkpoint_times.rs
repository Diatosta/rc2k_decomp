//! `saves::default_checkpoint_times` (0x47CA46): made-up best times with checkpoint times
//! for the championship's stages.
//!
//! For each stage of the championship's record table (counted into `seconds`): the ten entries of
//! the stage in the stage times table get the record's +0x18 + 24,000 and 0x9F more each, and
//! a driver, with the record's +4 as the stage for the checkpoints (`saves/default_times.rs`; its
//! caller has the checkpoint switch at 1 for this one, so the checkpoint times are guessed and some 9,000
//! pieces go to the game's log). Then the records are gone through once more as in
//! `default_championship_times`, but only the running sum is added up and cleared: nothing is done
//! with it. One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded. Refuses a
//! missing table and a record table without stages or without its end. Converted:
//! `rc2k_core::saves::defaults::championship_times` (a refusal now leaves nothing written).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::saves::allocate_best_times::count_stages;
use crate::saves::default_times::*;
use crate::system::owner::System;
use rc2k_core::saves::defaults::{Championship, RALLIES, championship_times};

// ---- game data ----

/// The first argument is not used: the stage comes from the record.
pub fn default_checkpoint_times(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    race: &Race,
    system: &System,
    _ebp: u32,
    log: Log,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    // Converted: `rc2k_core::saves::defaults::championship_times`.
    {
        let records_at = system.records.stage_tables.championship();
        let stages = count_stages(records_at)?;
        let count = records(records_at)?;
        let table = &crate::saves::best_times::TIME_TRIAL_TIMES;
        let (true, true) = (stages != 0, table.is_taken()) else {
            return Err(BadInput("no stages, or no table for their times").into());
        };
        let records: Vec<_> = (0..count as u32)
            .map(|n| read_record(records_at.row(n)?))
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
            crate::saves::default_times::with_checkpoints(screens),
        )?;
        apply(screens, race, &filled, &tables)?;
        Ok(())
    }
}
