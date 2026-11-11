//! `drive::play_race` (0x444CEE): "CALLING DOPLAYGAME - START" logged between rules, the log off
//! (`g_logEnabled` 0) for `drive::race_loop`, back on, "... - FINIS" between rules.
//!
//! Callers: `race::run_race`, 0x443F96, 0x44402F, `race::credits_race`, `net::lets_race`.
//! Refuses what `race_loop` refuses.

use crate::drive::race_parts::log_line;
use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;

pub fn play_race(race: &Race, o: Owners<'static>) -> Result<(), Refused> {
    log_line(c"");
    log_line(c"--------------------------");
    log_line(c"CALLING DOPLAYGAME - START");
    crate::logfile::write_line::set_enabled(0); // g_logEnabled
    { crate::drive::race_loop::race_loop(race, o.input, o.system, o.scene) }?;
    // Not the original's: for the debug UI's free play.
    crate::debug::screens::note_race();
    crate::logfile::write_line::set_enabled(1);
    log_line(c"CALLING DOPLAYGAME - FINIS");
    log_line(c"--------------------------");
    log_line(c"");
    Ok(())
}
