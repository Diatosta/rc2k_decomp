//! With the grades' mode (`driver_kind`) 0 or 2 the `stuck_count` is set to 0; otherwise it goes
//! up, and at 700 or more (unsigned) `physics::reset_car` puts the car back on the line.
//!
//! Refuses what `reset_car` refuses.

use crate::errors::Refused;
use crate::physics::step_families::ResetInputs;
use rc2k_core::physics::start::stuck_tick;

// Core: `rc2k_core::physics::start::stuck_tick`.
pub fn reset_when_stuck(
    c: &ResetInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let shared = c.shared;
    let mode = shared.car().driver_kind.get();
    let mut stuck = if mode == 2 || mode == 0 {
        0
    } else {
        shared.car().stuck_count.get()
    };
    let reset = stuck_tick(mode, &mut stuck);
    shared.car().stuck_count.set(stuck);
    if reset {
        crate::physics::reset_car::reset_car(c, scene)?;
    }
    Ok(())
}
