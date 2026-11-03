//! `menu::run_a8_championship` (0x4AA4D7): a whole A8 championship (event type 5,
//! `g_eventMode` 0x13), from its start or a saved game. `menu::run_championship`'s
//! driver (`run_championship::run_event`) with these differences: `clear_players_284`
//! before `dword_755254` 0; `g_eventMode` 0x13 (so the welcome is `menu::a8_welcome`) and
//! `dword_75AB40` 1; the section to start at is `dword_7554C4`; `dword_755970` is not set
//! back to 0; `menu::reset_single_event` where the other calls `menu::begin_event` (the
//! welcome record, the next section); no overall standings, overall stage or rally results
//! (only the class passes); the races are run even when `dword_755250` is 1;
//! `dword_755254` is not cleared after a stage; no `best_overall_scores` at the end; an
//! unknown record kind is fatal error 0x8070DBA9 "Invalid Level Type(A8)".
//!
//! In:  nothing
//! Out: every register as `menu::show_in_game_options` leaves the entry's (all kept)
//!
//! Callers: `saves::load_savegame` (`dword_7556D0` 0x13) and the five starters
//! (`menu::start_a8_*`). Refuses what its callees refuse, when they are
//! reached.

use crate::errors::Refused;
use crate::menu::run_championship::run_event;

pub fn run_a8_championship() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::championship::run` (`a8`).
    run_event(true)
}
