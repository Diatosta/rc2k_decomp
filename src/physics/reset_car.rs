//! `physics::reset_car` (0x43EFEF): the work car put back on the line. Its checkpoint
//! (`reset_checkpoint`, `g_car` +0x94) into `line_place`, `line_fraction` 0. With
//! `run_kind` 2 a free checkpoint is looked for: offsets d = 0, 1, .. 0x3F
//! (`reset_offset`), each tried as +d then -d (`reset_candidate` = the checkpoint + it), a
//! try failing when `physics::reset_point_blocked` gives CF for any of the blocks at
//! 0x7103CC, 0x7113AC, 0x71238C (tried in that order, the first CF ends it). The first
//! free one goes into `line_place`; with none, "cant reset car anywhere - waiting a sec"
//! is logged (`log_write_l1_nl`), +0x1E8 (`stuck_count`, the
//! stuck count) less 100, and it returns. Else `race::place_car_on_line`,
//! `physics::reset_car_on_line`, and +0xFAC (`reset_marker`) 10.
//!
//! Callers: `physics::reset_when_stuck`, `drive::reset_request`, `physics::find_clear_place`.
//! Refuses what the calls refuse.

use crate::errors::Refused;
use crate::physics::step_families::ResetInputs;
use crate::physics::tables_store::PhysicsTables;
use crate::race::vars_store::RaceVars;
use rc2k_core::physics::line_places::{RESET_TRIES, reset_offsets};

const NOWHERE_TEXT: &std::ffi::CStr = c"cant reset car anywhere - waiting a second"; // "cant reset car anywhere - waiting a sec"

/// One try: the candidate set; true when a block is in the way (CF).
fn blocked(
    vars: &RaceVars,
    physics_tables: &PhysicsTables,
    car_blocks: &crate::race::blocks_store::CarBlocks,
    scene: &crate::scene::owner::Scene,
) -> bool {
    physics_tables.reset_candidate.set(
        scene
            .lights
            .state
            .line_place
            .get()
            .wrapping_add(physics_tables.reset_offset.get()),
    );
    // Blocks 1 .. 3: the other cars.
    (1..4).any(|n| {
        crate::physics::reset_point_blocked::reset_point_blocked(
            physics_tables,
            vars,
            &car_blocks.cars[n],
        )
    })
}

// Converted: `rc2k_core::physics::line_places::reset_offsets`.
pub fn reset_car(c: &ResetInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let lights = &scene.lights;
    lights.state.line_place.set(c.car.reset_checkpoint.get());
    lights.state.line_fraction.set(0);
    if c.vars.run_kind.get() == 2 {
        let mut free = false;
        for offset in reset_offsets() {
            c.tables.reset_offset.set(offset);
            if !blocked(c.vars, c.tables, c.car_blocks, scene) {
                free = true;
                break;
            }
        }
        if !free {
            c.tables.reset_offset.set(RESET_TRIES);
            // `log_write_l1_nl`.
            crate::logfile::log_write::log_nul(&NOWHERE_TEXT, true, 1);
            c.car
                .stuck_count
                .set(c.car.stuck_count.get().wrapping_sub(100));
            return Ok(());
        }
        lights.state.line_place.set(c.tables.reset_candidate.get());
    }
    crate::race::start_place::place_car_on_line(c.shared, c.course, scene)?;
    crate::physics::reset_car_on_line::reset_car_on_line(c, scene)?;
    c.car.reset_marker.set(10);
    Ok(())
}
