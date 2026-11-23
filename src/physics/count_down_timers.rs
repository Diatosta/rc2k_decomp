//! The work car's step counters: `light_hold_steps`, `value_jump_steps`, `wall_hit_timer`,
//! `drive_cut_steps` and `hit_cooldown` count down by 1 when not 0; `stopped_steps` counts the
//! steps standing in neutral in mode 1 (reset to 0 below 1000 steps, with a switch set, with bits
//! 1 .. 4 of `parts_present` all set, or in gear; kept in other modes); `tyre_change_state` goes
//! up by one unless 0, -1 or 1; `top_speed` is raised to `engine_speed_scaled` (the highest so
//! far). Readings, all.

use crate::physics::car_store::Car;
use crate::race::vars_store::RaceVars;
use rc2k_core::physics::motion::{StepCounters, StepFacts, count_down};

/// One compare and copy of floats: the same at any precision.
// Core: `rc2k_core::physics::motion::count_down`.
pub(super) fn count_down_timers(car: &mut Car, vars: &RaceVars) {
    let mut c = StepCounters {
        first: [car.light_hold_steps, car.value_jump_steps],
        standing: car.stopped_steps,
        running: car.tyre_change_state,
        later: [car.wall_hit_timer, car.drive_cut_steps, car.hit_cooldown],
        highest: car.top_speed,
    };
    let facts = StepFacts {
        steps: car.run_ticks,
        switches: [car.finish_flag, car.go_stand_timer, car.started_stand_timer],
        mode: vars.run_kind.get(),
        bits: car.parts_present,
        gear: car.gear,
        figure: car.engine_speed_scaled,
    };
    count_down(&mut c, &facts);
    for (cell, x) in [
        (&mut car.light_hold_steps, c.first[0]),
        (&mut car.value_jump_steps, c.first[1]),
        (&mut car.stopped_steps, c.standing),
        (&mut car.tyre_change_state, c.running),
        (&mut car.wall_hit_timer, c.later[0]),
        (&mut car.drive_cut_steps, c.later[1]),
        (&mut car.hit_cooldown, c.later[2]),
    ] {
        if *cell != x {
            *cell = x;
        }
    }
    car.top_speed = c.highest;
}
