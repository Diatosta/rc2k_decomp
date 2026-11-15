//! The force feedback's clock: every 40 ms the effects are brought up to date.
//!
//! The tick count less the last time (`last_time`); under 40 ms nothing. Else the last time =
//! now, `rate` = 1000 / now held to 1000 (unsigned, as a float: it divides by the clock, not by
//! the time passed, so it is 1.0 once the clock is past a second; kept as is),
//! `steps` counts up, and `input::update_effects` runs. Refuses (before anything) a clock of 0
//! when due (it divides by it).

use crate::errors::{BadInput, Refused};

/// The force feedback's clock: once 40 ms have gone, the rate (a second over the gap,
/// held to 1000 ms) kept and `input::update_effects`.
pub fn step_effects_clock(
    input: &crate::input::owner::Input,
    sound: &crate::sound::owner::Sound,
    race: &crate::race::owner::Race,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let effects = &input.force_feedback;
    let last = effects.last_time.get();
    let gap =
        crate::system::tick_count::tick_count(&system.session.picture_clock).wrapping_sub(last);
    if (gap.wrapping_sub(0x28) as i32) < 0 {
        return Ok(());
    }
    let now = last.wrapping_add(gap);
    let divisor = now.min(1000);
    if divisor == 0 {
        return Err(BadInput("a clock of 0: the original divides by it").into());
    }
    effects.last_time.set(now);
    // `fild` / `fstp`: `as f32` at every precision.
    effects.rate.set(((1000 / divisor) as i32 as f32).to_bits());
    effects.steps.set(effects.steps.get().wrapping_add(1));
    crate::input::update_effects::update_effects(input, sound, race, system)
}
