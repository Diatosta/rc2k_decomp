//! The race's ticks for this frame. The clock in tenths
//! (`drive::race_clock_tick`); unless paused (`PhysicsTables.debug[1]`), `drive::race_tick` once for each
//! tenth gone (`race_tenths`, signed), stopping when the players are all out
//! (`race::players_all_out`).
//!
//! Refuses what its calls refuse.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn race_step(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    crate::drive::race_clock_tick::race_clock_tick(&race.shared, &o.system.session.picture_clock);
    if race.shared.physics_tables.get().debug[1].get() == 0 {
        rc2k_core::drive::race_ticks::run_ticks(
            race.shared.physics_tables.get().race_tenths.get(),
            |ecx| {
                crate::debug::timing::note_tick();
                { crate::drive::race_tick::race_tick(o, race) }?;
                crate::remote::sim_hash_step(race);
                let out = crate::race::frame::players_all_out(race, &o.net.race_slot)?;
                Ok::<_, Refused>((out, ecx))
            },
        )?;
    }
    let tables = race.shared.physics_tables.get();
    crate::race::blend::ticks_done(
        tables
            .race_clock_now
            .get()
            .wrapping_sub(tables.race_clock_base.get()),
        tables.debug[1].get() == 0,
    );
    Ok(())
}
