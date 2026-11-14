//! Puts the work car back on the line at the checkpoint nearest it: `race::nearest_checkpoint` into
//! `checkpoint`, `reset_checkpoint`, `last_pace_call`, `line_place` and `line_fraction`;
//! `race::place_car_on_line`, `race::reset_car_motion`; then `value_jump_steps` and
//! `light_hold_steps` = 8 and `reset_marker` = 10.
//!
//! Caller: `physics::reset_car`. Refuses what the three calls refuse.

use crate::errors::Refused;
use crate::physics::step_families::ResetInputs;

/// The nearest checkpoint (`race::nearest_checkpoint`), the car placed there
/// (`race::place_car_on_line`) and its motion reset (`race::reset_car_motion`).
pub fn reset_car_on_line(
    c: &ResetInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let lights = &scene.lights;
    let nearest =
        crate::race::start_place::nearest_checkpoint(c.physics, c.course, c.shared, scene)?;
    for cell in [
        &c.car.checkpoint,
        &c.car.reset_checkpoint,
        &c.car.last_pace_call,
    ] {
        cell.set(nearest);
    }
    for cell in [
        &lights.state.get().line_place,
        &lights.state.get().line_fraction,
    ] {
        cell.set(nearest);
    }
    crate::race::start_place::place_car_on_line(c.shared, c.course, scene)?;
    crate::race::car_starts::reset_car_motion(c, scene)?;
    c.car.value_jump_steps.set(8u32);
    c.car.light_hold_steps.set(8u32);
    c.car.reset_marker.set(10u32);
    Ok(())
}
