//! `menu::run_a8_championship`: a whole A8 championship (event type 5, event mode 0x13), from its
//! start or a saved game. `menu::run_championship`'s driver (`run_championship::run_event`) with
//! these differences: `clear_hud_bits` before `services` is cleared; the event mode is 0x13 (so the
//! welcome is `menu::a8_welcome`) and the class mode 1; the section to start at is
//! `start_section_a8`; `continuing_save` is not cleared; `menu::reset_single_event` where the other
//! calls `menu::begin_event` (the welcome record, the next section); no overall standings, overall
//! stage or rally results (only the class passes); the races are run even when `reset_times` is
//! set; `services` is not cleared after a stage; no `best_overall_scores` at the end; an unknown
//! record kind is fatal error 0x8070DBA9 "Invalid Level Type(A8)".
//!
//! Callers: `saves::load_savegame` (event mode 0x13) and the five starters (`menu::start_a8_*`).
//! Refuses what its callees refuse, when they are reached.

use crate::errors::Refused;
use crate::menu::run_championship::run_event;

pub fn run_a8_championship_in(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    run_event(o, true)
}
