//! `saves::default_checkpoint_times` (0x47CA46): made-up best times with checkpoint times
//! for the championship's stages.
//!
//! In:  nothing
//! Out: every register preserved
//! For each stage of the championship's record table (counted into `seconds`): the ten entries of
//! the stage in the table at `dword_7556E8` get the record's +0x18 + 24,000 and 0x9F more each, and
//! a driver, with the record's +4 as the stage for the checkpoints (`saves/default_times.rs`; its
//! caller has `dword_755A0C` at 1 for this one, so the checkpoint times are guessed and some 9,000
//! pieces go to the game's log). Then the records are gone through once more as in
//! `default_championship_times`, but only `dword_7559B0` is added up and cleared: nothing is done
//! with it. One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded. Refuses a
//! missing table and a record table without stages or without its end. Converted:
//! `rc2k_core::saves::defaults::championship_times` (a refusal now leaves nothing written).

use crate::errors::{BadInput, Refused};
use crate::memory::blocks::Place;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::saves::allocate_best_times::count_stages;
use crate::saves::default_times::*;
use rc2k_core::saves::defaults::{Championship, RALLIES, championship_times};

// ---- game data ----

// ---- Rust version ----

/// `ebp` is not used: the stage comes from the record.
pub fn default_checkpoint_times(_ebp: u32, log: Log) -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::defaults::championship_times`.
    {
        let records_at = crate::records::stages_store::STAGE_TABLES.championship();
        let stages = count_stages(records_at)?;
        let count = records(records_at)?;
        let table = CHAMP_VIEW.time_trial_best_times.get();
        let (true, Some(table)) = (stages != 0, table) else {
            return Err(BadInput("no stages, or no table for their times").into());
        };
        let records: Vec<_> = (0..count as u32)
            .map(|n| read_record(records_at + n * RECORD_LEN))
            .collect::<Result<_, _>>()?;
        let tables = [(Place::Owned(table), (stages * STAGE_LEN) as usize)];
        let filled = championship_times(
            &mut GameSource { log },
            &records,
            Championship::Checkpoints,
            [false; RALLIES],
            crate::saves::default_times::with_checkpoints(),
        )?;
        apply(&filled, &tables)?;
        Ok(())
    }
}
