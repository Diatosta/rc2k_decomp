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
//! Shadow mode runs the original alone (the clock moves, the effects are sent).
//! Self-test: a copy whose two calls go to stand-ins (the test's clock), against the
//! Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::sound::turn_parts::turn_entry;

/// dword_518084 the last time, dword_518088 the rate, dword_51808C the steps counted.
const LAST: usize = 0x518084;
const RATE: usize = 0x518088;
const STEPS: usize = 0x51808C;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

// ---- Rust version ----

pub fn step_effects_clock_in(r: &mut Regs) -> Result<(), Refused> {
    let mut c = *r;
    c.ebx = g(LAST);
    c.eax = crate::system::tick_count::tick_count();
    c.eax = c.eax.wrapping_sub(c.ebx);
    if (c.eax.wrapping_sub(0x28) as i32) < 0 {
        return Ok(());
    }
    c.ebx = c.ebx.wrapping_add(c.eax);
    let divisor = c.ebx.min(1000);
    if divisor == 0 {
        return Err(BadInput("a clock of 0: the original divides by it").into());
    }
    p(LAST, c.ebx);
    (c.eax, c.edx, c.ebx) = (1000 / divisor, 1000 % divisor, divisor);
    // `fild` / `fstp`: `as f32` at every precision.
    p(RATE, (c.eax as i32 as f32).to_bits());
    p(STEPS, g(STEPS).wrapping_add(1));
    crate::input::update_effects::update_effects(&mut c)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, step_effects_clock_in)
}
