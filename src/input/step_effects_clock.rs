//! `input::step_effects_clock` (0x40F257): the force feedback's clock: every 40 ms the
//! effects brought up to date.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! `system::tick_count` less the last time (`dword_518084`); under 40 ms (`js`)
//! nothing. Else the last time = now, `dword_518088` = 1000 / now held to 1000 (unsigned
//! `div`, as a float: the original divides by the clock, not by the time passed, so it
//! is 1.0 once the clock is past a second; notes' bugs), `dword_51808C` counts up, and
//! `input::update_effects` (its Rust version in the game).
//! One caller, `sound::race_turn`.
//! Refuses (before anything) a clock of 0 when due (the original's `div` traps).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};

// ---- Rust version ----

/// The force feedback's clock: once 40 ms have gone, the rate (a second over the gap,
/// held to 1000 ms) kept and `input::update_effects`.
pub fn step_effects_clock(input: &crate::input::owner::Input) -> Result<(), Refused> {
    let effects = &input.force_feedback;
    let last = effects.last_time.get();
    let gap = crate::system::tick_count::tick_count().wrapping_sub(last);
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
    crate::input::update_effects::update_effects(input)
}
