//! `drive::race_step` (0x4454D2): the race's ticks for this frame. The clock in tenths
//! (`drive::race_clock_tick`); unless paused (`dword_6138F8`), `drive::race_tick` once for each
//! tenth gone (`dword_613844`, signed), stopping when the players are all out
//! (`race::players_all_out`).
//!
//! Callers: `drive::race_loop`, `race::play_movie_loop`. Refuses what its calls refuse.

use crate::errors::Refused;
use crate::race::owner::Race;

// Converted: `rc2k_core::drive::race_ticks::run_ticks`.
pub fn race_step(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let _step = crate::write_watch::step("physics");
    let input = crate::input::owner::input();
    let sound = crate::sound::owner::sound();
    crate::drive::race_clock_tick::race_clock_tick(&race.shared);
    if race.shared.physics_tables.debug[1].get() == 0 {
        rc2k_core::drive::race_ticks::run_ticks(
            race.shared.physics_tables.race_tenths.get(),
            |ecx| {
                crate::debug::timing::note_tick();
                { crate::drive::race_tick::race_tick(race, sound, input, scene) }?;
                hash_if_asked(race);
                let out = crate::race::frame::players_all_out(race)?;
                Ok::<_, Refused>((out, ecx))
            },
        )?;
    }
    Ok(())
}

/// With `RC2K_SIM_HASH` (a check, as `RC2K_FAN_HASH` for the drawing): the car blocks and
/// the random state hashed after each tick and logged, so that two runs of a replay can be
/// compared tick by tick (the simulation must stay bit-identical).
fn hash_if_asked(race: &Race) {
    use std::hash::{Hash, Hasher};
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
    static ON: OnceLock<bool> = OnceLock::new();
    static TICKS: AtomicU32 = AtomicU32::new(0);
    if !*ON.get_or_init(|| std::env::var_os("RC2K_SIM_HASH").is_some()) {
        return;
    }
    let blocks = &race.replay.cars_and_replay.car_blocks;
    let mut bytes = vec![0u8; core::mem::size_of_val(blocks)];
    let _ = crate::pointers::Store::load(blocks, 0, &mut bytes);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    let tick = TICKS.fetch_add(1, Relaxed);
    crate::framework::log(&format!(
        "sim hash {tick}: {:016x}, random {:08x}",
        hasher.finish(),
        crate::math::next_random::STATE.get()
    ));
    // With `RC2K_SIM_CHUNKS` too: each 0x100 bytes' hash, to find which part of the blocks
    // differs between two runs (the held race's stray key was found so, at car 0's +0x400).
    if std::env::var_os("RC2K_SIM_CHUNKS").is_some() {
        let chunks: Vec<String> = bytes
            .chunks(0x100)
            .map(|c| {
                let mut h = std::collections::hash_map::DefaultHasher::new();
                c.hash(&mut h);
                format!("{:04x}", h.finish() & 0xFFFF)
            })
            .collect();
        crate::framework::log(&format!("sim chunks {tick}: {}", chunks.join(" ")));
    }
}
