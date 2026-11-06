//! `saves::default_championship_times` (0x47C973): made-up best times for the
//! championship's stages and rallies.
//!
//! In:  ebp (passed on as the stage to `default_time_entry`, which only uses it while
//! `dword_755A0C` is 1)
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

use crate::errors::{BadInput, Refused};
use crate::saves::allocate_best_times::count_stages;
use crate::saves::default_times::*;
use crate::saves::owner::saves;
use rc2k_core::saves::defaults::{Championship, RALLIES, championship_times};

// ---- game data ----

// ---- Rust version ----

pub fn default_championship_times(ebp: u32, log: Log) -> Result<(), Refused> {
    let screens = &crate::menu::owner::menus().screens;
    // Converted: `rc2k_core::saves::defaults::championship_times`.
    {
        let records_at = crate::records::stages_store::STAGE_TABLES.championship();
        let stages = count_stages(records_at)?;
        let count = records(records_at)?;
        let table = &crate::saves::best_times::STAGE_TIMES;
        let (true, true) = (stages != 0, table.is_taken()) else {
            return Err(BadInput("no stages, or no table for their times").into());
        };
        let records: Vec<_> = (0..count as u32)
            .map(|n| read_record(records_at.row(n)?))
            .collect::<Result<_, _>>()?;
        let mut tables = vec![Target::Stages(table)];
        // Every rally has its table.
        let present = [true; RALLIES];
        for rally in 0..RALLIES {
            tables.push(Target::Rally(saves().rally_times.table(rally)?));
        }
        let filled = championship_times(
            &mut GameSource { log },
            &records,
            Championship::Times { stage: ebp },
            present,
            with_checkpoints(screens),
        )?;
        apply(screens, &filled, &tables)?;
        Ok(())
    }
}
