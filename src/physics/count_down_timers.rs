//! `physics::count_down_timers` (0x43D4B5): the work car's step counters:
//! `dword_71AF8C`, `dword_71AF88`, `dword_71A1CC`, `dword_71A1A8`, `dword_71A438` less 1
//! when not 0; `dword_71A240` counts the steps standing in neutral in mode 1 (0 below 1000
//! steps, with a switch of 0x71A078 .. 0x71A080 set, with bits 1 .. 4 of `dword_71A270` all
//! set, or in gear; kept in other modes); `dword_71A244` on by one unless 0, -1 or 1;
//! `flt_71A5D4` raised to `flt_71AEE0` (the highest so far). Readings, all.
//!
//! In:  nothing
//! Out: eax `dword_71A270` & 0x1E when that is looked at, else kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B70A) and `sub_421848` (the car specification's run).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::car_store::CAR;
use crate::race::vars_store::RACE_VARS;
use crate::session::Cell32;
use rc2k_core::physics::motion::{StepCounters, StepFacts, count_down};

/// The converted version (`rc2k_core::physics::motion`) on the variables' cells `v`; `r` as
/// the original leaves (eax) are read by none of its callers.
// Converted: `rc2k_core::physics::motion::count_down`.
pub fn count_at(v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    let mut c = StepCounters {
        first: [get(0), get(1)],
        standing: get(9),
        running: get(10),
        later: [get(11), get(12), get(15)],
        highest: f32::from_bits(get(14)),
    };
    let facts = StepFacts {
        steps: get(2),
        switches: [get(3), get(4), get(5)],
        mode: get(6),
        bits: get(7),
        gear: get(8),
        figure: f32::from_bits(get(13)),
    };
    count_down(&mut c, &facts);
    for (k, x) in [
        (0, c.first[0]),
        (1, c.first[1]),
        (9, c.standing),
        (10, c.running),
        (11, c.later[0]),
        (12, c.later[1]),
        (14, c.highest.to_bits()),
        (15, c.later[2]),
    ] {
        if get(k) != x {
            put(k, x);
        }
    }
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &CAR.light_hold_steps as &dyn Cell32,
        &CAR.value_jump_steps,
        &CAR.run_ticks,
        &CAR.finish_flag,
        &CAR.go_stand_timer,
        &CAR.started_stand_timer,
        &RACE_VARS.run_kind,
        &CAR.parts_present,
        &CAR.gear,
        &CAR.stopped_steps,
        &CAR.tyre_change_state,
        &CAR.wall_hit_timer,
        &CAR.drive_cut_steps,
        &CAR.engine_speed_scaled,
        &CAR.top_speed,
        &CAR.hit_cooldown,
    ]
}

/// The Rust version (one compare and copy of floats: the same at any precision); `r` as
/// the registers it leaves are read by none of its callers.
pub fn count_down_timers() {
    count_at(&game())
}
