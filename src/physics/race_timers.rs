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
use crate::race::owner::Race;

// Converted: `rc2k_core::physics::splits::{timer_rate, timer_run}`.
pub fn race_timers(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let sound = o.sound;
    let (physics, numbers, league) = (&race.physics, &race.course.numbers, race.course.league());
    let (frame_work, panel_and_overlay) = (
        &race.frame.frame_work.get(),
        &race.frame.panel_and_overlay.get(),
    );
    let (shared, car_blocks) = (&race.shared, race.car_blocks());
    use rc2k_core::physics::splits::{TimerRun, timer_rate, timer_run};
    shared.physics_tables.weight.set(1.0f32);
    let finished = shared.car().finish_flag.get() != 0;
    let factor = if finished {
        physics.split_tables().timer_factor.get()
    } else {
        0.0
    };
    shared
        .physics_tables
        .weight
        .set(timer_rate(finished, factor));
    let run = timer_run(shared.vars.run_kind.get());
    if run == TimerRun::None {
        return Ok(());
    }
    count_start_second(shared.car());
    if run == TimerRun::Start {
        count_down_start(frame_work, &shared.work, shared.car(), sound)?;
        take_split_counted(
            o,
            physics,
            league,
            frame_work,
            panel_and_overlay,
            shared,
            car_blocks,
        )?;
        start_go_notice(frame_work, &shared.work, shared.car(), sound, o.menus)?;
    } else {
        take_split(
            o,
            physics.split_tables(),
            league,
            panel_and_overlay,
            shared,
            car_blocks,
        )?;
    }
    count_stop_timers(shared.car(), numbers);
    mark_car_started(shared.car());
    Ok(())
}
