//! `race::play_movie`: "shaunplayrendermovie - in" logged, the clock restarted
//! (`scene::restart_clock`, `scene_clock` into `race_clock_base`), the replay played
//! (`race::play_movie_loop`), "shaunplayrendermovie - out" logged.

use crate::errors::Refused;
use crate::race::owner::Race;

const IN_TEXT: &std::ffi::CStr = c"shaunplayrendermovie - in";
const OUT_TEXT: &std::ffi::CStr = c"shaunplayrendermovie - out";

/// Plays the replay between the two log lines.
pub fn play_movie(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    crate::logfile::log_write::log_nul(&IN_TEXT, false, 1);
    crate::drive::race_parts::restart_clock(race, o.system);
    race.shared
        .physics_tables
        .get()
        .race_clock_base
        .set(race.shared.physics_tables.get().scene_clock.get());
    crate::race::play_movie_loop::play_movie_loop(race, o)?;
    crate::logfile::log_write::log_nul(&OUT_TEXT, true, 1);
    Ok(())
}
