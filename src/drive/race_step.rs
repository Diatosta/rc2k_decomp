//! `drive::race_step` (0x4454D2): the race's ticks for this frame. The clock in tenths
//! (`drive::race_clock_tick`); unless paused (`dword_6138F8`), `drive::race_tick` once for
//! each tenth gone (`dword_613844`, signed), stopping when the players are all out
//! (`race::players_all_out`).
//!
//! In:  nothing
//! Out: ecx the tenths left, the rest as the ticks leave them; no flags read
//!
//! Callers: `drive::race_loop`, `sub_4607A5`. It begins with a call: hooked by a private
//! copy of its 0x34 bytes (four calls relocated) and a hand-made patch.
//! Refuses what its calls refuse. Shadow mode runs the original alone (the clock moves).
//! Self-test: skipped (its parts carry their own; the race in play checks it).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::g;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;

/// The Rust version.
// Converted: `rc2k_core::drive::race_ticks::run_ticks`.
pub unsafe fn race_step(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    unsafe { crate::drive::race_clock_tick::race_clock_tick() }.apply(&mut r);
    if g(0x6138F8) == 0 {
        r.ecx = rc2k_core::drive::race_ticks::run_ticks(g(0x613844), |ecx| {
            r.ecx = ecx;
            unsafe { crate::drive::race_tick::race_tick(&r) }?.apply(&mut r);
            r.ecx = ecx;
            let out = crate::drive::race_parts::players_all_out(&mut r)?;
            Ok::<_, Refused>((out, r.ecx))
        })?;
    }
    Ok(left_of(&r))
}
