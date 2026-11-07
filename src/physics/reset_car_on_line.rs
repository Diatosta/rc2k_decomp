//! `physics::reset_car_on_line` (0x43FB00): the work car put back on the line at the
//! checkpoint nearest it: `race::nearest_checkpoint` into `dword_71A070`, `6C`, `74`
//! (`g_car` +0x98, +0x94, +0x9C), `dword_6365E4` and `dword_6365EC`;
//! `race::place_car_on_line`, `race::reset_car_motion`; then +0xFB0 and +0xFB4
//! (`dword_71AF88`, `8C`) 8 and +0xFAC (`dword_71AF84`) 10.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: as `reset_car_motion` leaves them; no flags read (the caller's next is a `mov`)
//!
//! Caller: `physics::reset_car` (its end). Refuses what the three calls refuse.

use crate::errors::Refused;
use crate::race::owner::Race;

/// The Rust version: the nearest checkpoint (`race::nearest_checkpoint`), the car placed
/// there (`race::place_car_on_line`) and its motion reset (`race::reset_car_motion`).
/// Those take no register (they are called with none); the registers left are read by
/// none of the callers.
pub fn reset_car_on_line(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let lights = &scene.lights;
    let nearest = crate::race::start_place::nearest_checkpoint(race, scene)?;
    for cell in [
        &race.shared.car.checkpoint,
        &race.shared.car.reset_checkpoint,
        &race.shared.car.last_pace_call,
    ] {
        cell.set(nearest);
    }
    for cell in [&lights.state.line_place, &lights.state.line_fraction] {
        cell.set(nearest);
    }
    crate::race::start_place::place_car_on_line(race, &race.course, scene)?;
    crate::race::car_starts::reset_car_motion(race)?;
    race.shared.car.value_jump_steps.set(8u32);
    race.shared.car.light_hold_steps.set(8u32);
    race.shared.car.reset_marker.set(10u32);
    Ok(())
}
