//! The race's ticks for this frame. The clock in tenths
//! (`drive::race_clock_tick`); unless paused (`PhysicsRun.debug[1]`), `drive::race_tick` once for each
//! tenth gone (`race_tenths`, signed), stopping when the players are all out
//! (`race::players_all_out`).
//!
//! Refuses what its calls refuse.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn race_step(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    crate::drive::race_clock_tick::race_clock_tick(&race.shared, &o.system.session.picture_clock);
    if race.shared.run().debug[1] == 0 {
        let tenths = race.shared.run().race_tenths;
        rc2k_core::drive::race_ticks::run_ticks(tenths, |ecx| {
            crate::debug::timing::note_tick();
            { crate::drive::race_tick::race_tick(o, race) }?;
            crate::remote::sim_hash_step(race);
            let out = crate::race::frame::players_all_out(race, &o.net.race_slot)?;
            Ok::<_, Refused>((out, ecx))
        })?;
    }
    let (clock_gap, running) = {
        let run = race.shared.run();
        (
            run.race_clock_now.wrapping_sub(run.race_clock_base),
            run.debug[1] == 0,
        )
    };
    crate::race::blend::ticks_done(race, clock_gap, running);
    Ok(())
}
