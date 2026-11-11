//! `physics::race_timers` (0x436790): the race's timers for the work car, every physics
//! step. `weight` 1.0, or `timer_factor` once finished (+0xA8). Then by the race mode
//! (`run_kind`): 2: `physics::count_start_second`, `physics::count_down_start`,
//! `physics::take_split_counted`, `physics::start_go_notice`,
//! `physics::count_stop_timers`, `physics::mark_car_started`; 0, 1, 3:
//! `count_start_second`, `physics::take_split`, `count_stop_timers`, `mark_car_started`;
//! else nothing.
//!
//! Caller: `physics::car_physics_step` (0x43B64E). Refuses what the steps refuse; what was written
//! is put back.
use crate::errors::Refused;
use crate::physics::count_down_start::count_down_start;
use crate::physics::count_start_second::count_start_second;
use crate::physics::count_stop_timers::count_stop_timers;
use crate::physics::mark_car_started::mark_car_started;
use crate::physics::start_go_notice::start_go_notice;
use crate::physics::take_split::take_split;
use crate::physics::take_split_counted::take_split_counted;
use crate::race::frame_store::FrameWork;
use crate::race::groups_store::PanelAndOverlay;
use crate::race::league_store::League;
use crate::race::numbers_store::Numbers;
use crate::race::owner_physics::Physics;
use crate::race::owner_shared::RaceShared;

// Converted: `rc2k_core::physics::splits::{timer_rate, timer_run}`.
pub fn race_timers(
    physics: &Physics,
    numbers: &Numbers,
    league: &League,
    frame_work: &FrameWork,
    panel_and_overlay: &PanelAndOverlay,
    shared: &RaceShared,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    use rc2k_core::physics::splits::{TimerRun, timer_rate, timer_run};
    shared.physics_tables.weight.set_bits(1.0f32.to_bits());
    let finished = shared.car.finish_flag.get() != 0;
    let factor = if finished {
        physics.split_tables.timer_factor.bits()
    } else {
        0
    };
    shared
        .physics_tables
        .weight
        .set_bits(timer_rate(finished, f32::from_bits(factor)).to_bits());
    let run = timer_run(shared.vars.run_kind.bits());
    if run == TimerRun::None {
        return Ok(());
    }
    count_start_second(&shared.car);
    if run == TimerRun::Start {
        count_down_start(frame_work, &shared.work, &shared.car, sound)?;
        take_split_counted(
            physics,
            league,
            frame_work,
            panel_and_overlay,
            shared,
            sound,
        )?;
        start_go_notice(frame_work, &shared.work, &shared.car, sound)?;
    } else {
        take_split(&physics.split_tables, league, panel_and_overlay, shared)?;
    }
    count_stop_timers(&shared.car, numbers);
    mark_car_started(&shared.car);
    Ok(())
}
