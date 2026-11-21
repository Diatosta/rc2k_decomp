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
pub(super) fn count_down_timers(car: &Car, vars: &RaceVars) {
    let mut c = StepCounters {
        first: [car.light_hold_steps.get(), car.value_jump_steps.get()],
        standing: car.stopped_steps.get(),
        running: car.tyre_change_state.get(),
        later: [
            car.wall_hit_timer.get(),
            car.drive_cut_steps.get(),
            car.hit_cooldown.get(),
        ],
        highest: car.top_speed.get(),
    };
    let facts = StepFacts {
        steps: car.run_ticks.get(),
        switches: [
            car.finish_flag.get(),
            car.go_stand_timer.get(),
            car.started_stand_timer.get(),
        ],
        mode: vars.run_kind.get(),
        bits: car.parts_present.get(),
        gear: car.gear.get(),
        figure: car.engine_speed_scaled.get(),
    };
    count_down(&mut c, &facts);
    for (cell, x) in [
        (&car.light_hold_steps, c.first[0]),
        (&car.value_jump_steps, c.first[1]),
        (&car.stopped_steps, c.standing),
        (&car.tyre_change_state, c.running),
        (&car.wall_hit_timer, c.later[0]),
        (&car.drive_cut_steps, c.later[1]),
        (&car.hit_cooldown, c.later[2]),
    ] {
        if cell.get() != x {
            cell.set(x);
        }
    }
    car.top_speed.set(c.highest);
}
