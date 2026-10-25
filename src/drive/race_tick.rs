//! `drive::race_tick` (0x4455A9, three chunks): one tick of the race. Fatal error
//! 0x80600655 (no message) when the first car block's +0x260 is 0 (its third chunk;
//! the second is the shared fatal reporter). Free roaming (`dword_621884` 0):
//! `drive::simple_car_step`. Else the tick count `dword_6235F8` + 1 and
//! `drive::replay_step` every tick, or with `dword_63DB84` set every second tick, and
//! past 1 every fourth.
//!
//! In:  nothing
//! Out: the registers as the step leaves them; no flags read
//!
//! Caller: `drive::race_step`. Raises 0x80600655 as the original; refuses what the steps
//! refuse.

use crate::errors::{FatalError, Refused};
use crate::physics::flow::g;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;

/// The Rust version.
// Converted: `rc2k_core::drive::race_ticks::{tick_runs, NO_CAR_FATAL}`.
pub unsafe fn race_tick() -> Result<(), Refused> {
    if CAR_BLOCKS.cars[0].probe_count.bits() == 0 {
        return Err(FatalError {
            code: rc2k_core::drive::race_ticks::NO_CAR_FATAL,
            message: "",
        }
        .into());
    }
    if g(0x621884) == 0 {
        return unsafe { crate::drive::simple_car_step::simple_car_step() };
    }
    RACE_VARS.race_ticks.set(g(0x6235F8).wrapping_add(1));
    if !rc2k_core::drive::race_ticks::tick_runs(
        RACE_VARS.race_ticks.get(),
        RACE_VIEW.after_credits.get(),
    ) {
        return Ok(());
    }
    unsafe { crate::drive::replay_step::replay_step() }
}
