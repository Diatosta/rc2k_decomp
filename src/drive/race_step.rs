//! `drive::race_step` (0x4454D2): the race's ticks for this frame. The clock in tenths
//! (`drive::race_clock_tick`); unless paused (`dword_6138F8`), `drive::race_tick` once for
//! each tenth gone (`dword_613844`, signed), stopping when the players are all out
//! (`race::players_all_out`).
//!
//! In:  nothing
//! Out: ecx the tenths left, the rest as the ticks leave them; no flags read
//!
//! Callers: `drive::race_loop`, `sub_4607A5`. Refuses what its calls refuse.

use crate::errors::Refused;
use crate::physics::tables_store::PHYSICS;

/// The Rust version. The registers it leaves are read by no caller (liveness: edi, edx
/// and esi after `drive::race_loop`'s call have no read point but the false ones of
/// `imul` and the physics; `race::play_movie_loop` has only DF live after it).
// Converted: `rc2k_core::drive::race_ticks::run_ticks`.
pub fn race_step() -> Result<(), Refused> {
    crate::drive::race_clock_tick::race_clock_tick();
    if PHYSICS.debug[1].get() == 0 {
        rc2k_core::drive::race_ticks::run_ticks(PHYSICS.race_tenths.get(), |ecx| {
            crate::debug::timing::note_tick();
            { crate::drive::race_tick::race_tick() }?;
            let out = crate::race::frame::players_all_out()?;
            Ok::<_, Refused>((out, ecx))
        })?;
    }
    Ok(())
}
