//! `physics::reset_car` (0x43EFEF): the work car put back on the line. Its checkpoint
//! (`dword_71A06C`, `g_car` +0x94) into `dword_6365E4`, `dword_6365EC` 0. With
//! `dword_62389C` 2 a free checkpoint is looked for: offsets d = 0, 1, .. 0x3F
//! (`dword_614C4C`), each tried as +d then -d (`dword_614C50` = the checkpoint + it), a
//! try failing when `physics::reset_point_blocked` gives CF for any of the blocks at
//! 0x7103CC, 0x7113AC, 0x71238C (tried in that order, the first CF ends it). The first
//! free one goes into `dword_6365E4`; with none, "cant reset car anywhere - waiting a sec"
//! is logged (`log_write_l1_nl`, inside `pusha` / `popa`), +0x1E8 (`dword_71A1C0`, the
//! stuck count) less 100, and it returns. Else `race::place_car_on_line`,
//! `physics::reset_car_on_line`, and +0xFAC (`dword_71AF84`) 10.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: as the last call leaves them (eax the last try's checkpoint and esi the last
//!      block when none was free); no flags read (every caller returns next)
//!
//! Callers: `physics::reset_when_stuck`, `sub_43A0D3`, `sub_43BDD1`. Refuses what the calls
//! refuse.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner::Race;
use crate::race::vars_store::RACE_VARS;
use crate::scene::owner::scene;
use rc2k_core::physics::line_places::{RESET_TRIES, reset_offsets};

const NOWHERE_TEXT: &std::ffi::CStr = c"cant reset car anywhere - waiting a second"; // "cant reset car anywhere - waiting a sec"

/// One try: the candidate set; true when a block is in the way (CF).
fn blocked() -> bool {
    PHYSICS.reset_candidate.set(
        scene()
            .lights
            .state
            .line_place
            .get()
            .wrapping_add(PHYSICS.reset_offset.get()),
    );
    // Blocks 1 .. 3: the other cars.
    (1..4).any(|n| {
        crate::physics::reset_point_blocked::reset_point_blocked(
            crate::race::blocks_store::block_addr(n),
        )
    })
}

/// The Rust version. The registers it leaves are read by none of its callers.
// Converted: `rc2k_core::physics::line_places::reset_offsets`.
pub fn reset_car(race: &Race) -> Result<(), Refused> {
    let lights = &scene().lights;
    lights.state.line_place.set(CAR.reset_checkpoint.get());
    lights.state.line_fraction.set(0);
    if RACE_VARS.run_kind.get() == 2 {
        let mut free = false;
        for offset in reset_offsets() {
            PHYSICS.reset_offset.set(offset);
            if !blocked() {
                free = true;
                break;
            }
        }
        if !free {
            PHYSICS.reset_offset.set(RESET_TRIES);
            // `log_write_l1_nl`.
            crate::logfile::log_write::log_nul(&NOWHERE_TEXT, true, 1);
            CAR.stuck_count.set(CAR.stuck_count.get().wrapping_sub(100));
            return Ok(());
        }
        lights.state.line_place.set(PHYSICS.reset_candidate.get());
    }
    crate::race::start_place::place_car_on_line(&race.course)?;
    crate::physics::reset_car_on_line::reset_car_on_line(race)?;
    CAR.reset_marker.set(10);
    Ok(())
}
