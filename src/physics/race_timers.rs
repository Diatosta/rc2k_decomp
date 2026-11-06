//! `physics::race_timers` (0x436790): the race's timers for the work car, every physics
//! step. `flt_614170` 1.0, or `flt_601028` once finished (+0xA8). Then by the race mode
//! (`dword_62389C`): 2: `physics::count_start_second`, `physics::count_down_start`,
//! `physics::take_split_counted`, `physics::start_go_notice`,
//! `physics::count_stop_timers`, `physics::mark_car_started`; 0, 1, 3:
//! `count_start_second`, `physics::take_split`, `count_stop_timers`, `mark_car_started`;
//! else nothing.
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read (the caller goes on with
//!      calls)
//!
//! Caller: `sub_43B635` (0x43B64E). Refuses what the steps refuse; what was written is put
//! back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
use crate::errors::Refused;
use crate::physics::count_down_start::count_down_start;
use crate::physics::count_start_second::count_start_second;
use crate::physics::count_stop_timers::count_stop_timers;
use crate::physics::mark_car_started::mark_car_started;
use crate::physics::start_go_notice::start_go_notice;
use crate::physics::take_split::take_split;
use crate::physics::take_split_counted::take_split_counted;
use crate::race::owner::Race;

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::splits::{timer_rate, timer_run}`.
pub fn race_timers(race: &Race) -> Result<(), Refused> {
    use rc2k_core::physics::splits::{TimerRun, timer_rate, timer_run};
    crate::race::owner::race()
        .shared
        .physics_tables
        .weight
        .set_bits(1.0f32.to_bits());
    let finished = crate::race::owner::race().shared.car.finish_flag.get() != 0;
    let factor = if finished {
        race.physics.split_tables.timer_factor.bits()
    } else {
        0
    };
    crate::race::owner::race()
        .shared
        .physics_tables
        .weight
        .set_bits(timer_rate(finished, f32::from_bits(factor)).to_bits());
    let run = timer_run(crate::race::owner::race().shared.vars.run_kind.bits());
    if run == TimerRun::None {
        return Ok(());
    }
    count_start_second();
    if run == TimerRun::Start {
        count_down_start(&race.frame)?;
        take_split_counted(race)?;
        start_go_notice(&race.frame)?;
    } else {
        take_split(race)?;
    }
    count_stop_timers(&race.course);
    mark_car_started();
    Ok(())
}
