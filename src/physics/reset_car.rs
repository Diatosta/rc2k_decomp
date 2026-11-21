//! Puts the work car back on the line. Its checkpoint (`reset_checkpoint`) goes into `line_place`,
//! `line_fraction` = 0. With `run_kind` 2 a free checkpoint is looked for: offsets d = 0, 1, ..
//! 0x3F (`reset_offset`), each tried as +d then -d (`reset_candidate` = the checkpoint + it), a try
//! failing when `physics::reset_point_blocked` is true for any of the three other car blocks (tried
//! in that order, the first true ends it). The first free one goes into `line_place`; with none,
//! "cant reset car anywhere - waiting a sec" is logged (through `logfile::log_write`),
//! `stuck_count` is reduced by 100 and it returns. Otherwise `race::place_car_on_line`,
//! `physics::reset_car_on_line` and `reset_marker` = 10.
//!
//! Refuses what the calls refuse.

use crate::errors::Refused;
use crate::physics::step_families::ResetInputs;
use crate::physics::tables_store::PhysicsTables;
use crate::race::vars_store::RaceVars;
use rc2k_core::physics::line_places::{RESET_TRIES, reset_offsets};

const NOWHERE_TEXT: &std::ffi::CStr = c"cant reset car anywhere - waiting a second";

/// One try: the candidate set; true when a block is in the way.
fn blocked(
    vars: &RaceVars,
    physics_tables: &PhysicsTables,
    car_blocks: &crate::race::blocks_store::CarBlocks,
    scene: &crate::scene::owner::Scene,
) -> bool {
    physics_tables.reset_candidate.set(
        scene
            .line_and_shake()
            .line_place
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

// Core: `rc2k_core::physics::line_places::reset_offsets`.
pub fn reset_car(c: &ResetInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    scene.line_and_shake_mut().line_place = c.car.reset_checkpoint.get();
    scene.line_and_shake_mut().line_fraction = 0;
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
            // The log line, level 1.
            crate::logfile::log_write::log_nul(&NOWHERE_TEXT, true, 1);
            c.car
                .stuck_count
                .set(c.car.stuck_count.get().wrapping_sub(100));
            return Ok(());
        }
        scene.line_and_shake_mut().line_place = c.tables.reset_candidate.get();
    }
    crate::race::start_place::place_car_on_line(c.shared, c.course, scene)?;
    crate::physics::reset_car_on_line::reset_car_on_line(c, scene)?;
    c.car.reset_marker.set(10);
    Ok(())
}
