//! "CALLING DOPLAYGAME - START" logged between rules, the log off for `drive::race_loop`, back
//! on, "... - FINIS" between rules. Refuses what `race_loop` refuses.

use crate::drive::race_parts::log_line;
use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;

pub fn play_race(race: &Race, o: Owners<'static>) -> Result<(), Refused> {
    log_line(c"");
    log_line(c"--------------------------");
    log_line(c"CALLING DOPLAYGAME - START");
    crate::logfile::write_line::set_enabled(0);
    { crate::drive::race_loop::race_loop(race, o) }?;
    // For the debug UI's free play.
    crate::debug::screens::note_race();
    crate::logfile::write_line::set_enabled(1);
    log_line(c"CALLING DOPLAYGAME - FINIS");
    log_line(c"--------------------------");
    log_line(c"");
    Ok(())
}
