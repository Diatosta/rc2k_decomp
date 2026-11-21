//! Drives the network race: clears the stored cars' outputs, sets the HUD, plays the race while
//! recording its replay, keeps the HUD; then returns this machine's car's race time and whether it did not finish.

use crate::errors::Refused;
use crate::owners::Owners;

/// This machine's car's time (`stage_time`) and whether it did not finish
/// (`stop_state` is -1: `u32::MAX`, else 0).
pub fn lets_race(o: Owners<'static>) -> Result<(u32, u32), Refused> {
    let race = o.race;
    race.shared.view.get().after_credits.set(0);
    // The eight stored cars' `stop_state` (out).
    for car in &race.car_blocks().cars[..8] {
        car.stop_state.set(0);
    }
    crate::net::set_hud_options::set_hud_options(race, o.net, o.gfx)?;
    // The race is recorded as a single race is: every car's inputs, the cars' state each frame.
    let kept_mode = race.course.replay_mode();
    race.course.set_replay_mode(1);
    crate::race::race_end::record_replay_header(race);
    crate::race::replay::open_replay(race, o)?;
    crate::net::left::begin_race();
    let played = { crate::drive::play_race::play_race(race, o) };
    crate::net::left::end_race();
    crate::race::replay::close_replay(race, o.system)?;
    race.course.set_replay_mode(kept_mode);
    played?;
    crate::net::keep_hud_options::keep_hud_options(&race.shared, o.net)?;
    let block = race
        .shared
        .race_car(race.shared.view.get().local_car.get() as usize)?;
    // The logic is in `rc2k_core::net::record::not_finished`.
    Ok((
        block.stage_time.get(),
        rc2k_core::net::record::not_finished(block.stop_state.get()),
    ))
}
