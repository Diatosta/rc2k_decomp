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
                hash_if_asked(race);
                let out = crate::race::frame::players_all_out(race, &o.net.race_slot)?;
                Ok::<_, Refused>((out, ecx))
            },
        )?;
    }
    Ok(())
}

/// With `RC2K_SIM_HASH` (a check, as `RC2K_FAN_HASH` for the drawing; in the builds with the
/// `remote` feature): the car blocks and the random state hashed after each tick and logged,
/// so that two runs of a replay can be compared tick by tick (the simulation must stay
/// bit-identical). The blocks are hashed by field path and value (`drive::sim_hash`), so the
/// records' layout does not matter.
#[cfg(any(test, feature = "remote"))]
fn hash_if_asked(race: &Race) {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
    static ON: OnceLock<bool> = OnceLock::new();
    static TICKS: AtomicU32 = AtomicU32::new(0);
    if !*ON.get_or_init(|| std::env::var_os("RC2K_SIM_HASH").is_some()) {
        return;
    }
    let mut groups = crate::drive::sim_hash::car_blocks_groups(race.shared.car_blocks.get());
    let staging = race.replay.staging();
    let mut bytes = vec![0; core::mem::size_of_val(staging)];
    if crate::cells::load_cells(staging, 0, &mut bytes).is_err() {
        return;
    }
    groups.push(crate::drive::sim_hash::staging_group(&bytes));
    let tick = TICKS.fetch_add(1, Relaxed);
    crate::framework::log(&format!(
        "sim hash {tick}: {:016x}, random {:08x}",
        crate::drive::sim_hash::total(&groups),
        crate::math::next_random::STATE.get()
    ));
    // With `RC2K_SIM_CHUNKS` too: each car block's hash, to find which part of the blocks
    // differs between two runs.
    if std::env::var_os("RC2K_SIM_CHUNKS").is_some() {
        let chunks: Vec<String> = groups
            .iter()
            .map(|(name, hash)| format!("{name}={:04x}", hash & 0xFFFF))
            .collect();
        crate::framework::log(&format!("sim chunks {tick}: {}", chunks.join(" ")));
    }
}

#[cfg(not(any(test, feature = "remote")))]
fn hash_if_asked(_race: &Race) {}
