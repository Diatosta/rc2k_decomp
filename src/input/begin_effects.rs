//! `input::begin_effects` (0x40F29D): the race's force feedback begun: if any sound object
//! is in use (`g_raceSoundObjectCount`, a race is on) both records' effects asked to start
//! (`input::start_effects`); the clock (`system::tick_count`) kept as the effects' last
//! step (`g_effectsLastTime`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs`. One caller, `input::device_command` (command 3, the race's
//! start). Its calls through `part`. Shadow mode runs the original alone. Self-test: a
//! copy whose two variables are the test's and whose calls go to the noting stand-ins
//! (the clock the test's), against the Rust version, with a count of 0 and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::input::effects::{LAST_TIME, Part, START, TICK, effects_entry};
use crate::sound::turn_parts::COUNT;

/// The Rust version on the variables at `count` and `last` (addresses).
pub fn begin_effects_with(r: &mut Regs, count: u32, last: u32, part: Part) -> Result<(), Refused> {
    let mut g = *r;
    unsafe {
        if (count as *const u32).read_volatile() != 0 {
            part(START, &mut g)?;
        }
        part(TICK, &mut g)?;
        (last as *mut u32).write_volatile(g.eax);
    }
    Ok(())
}

/// The Rust version.
pub fn begin_effects(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    begin_effects_with(r, a(COUNT), a(LAST_TIME), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    effects_entry(r, begin_effects)
}
