//! `sound::step_race_clock` (0x40F20E): says whether a turn of the race's sounds is due.
//!
//! In:  nothing
//! Out: carry set = not yet; every register preserved
//! With fewer than 20 ms since the last turn (unsigned): carry set, nothing stored.
//! Else the last turn's time moves on by what has passed; that, held to 1000, is kept
//! as the turn's milliseconds, 1000 / it as the turns a second (a float), the turns
//! counted go up by one, and the carry is clear (`sound/race_clock.rs`). The zero flag
//! is clear either way.
//! One caller, `sub_40D5F3` (the race's sounds, every frame), which does `jb`.
//! Shadow mode runs the original alone (the clock moves) and checks what it left
//! against the Rust version with a clock that fits.
//! Self-test: a redirected copy with the test's clock.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::sound::race_clock::step_race_clock;
use crate::system::tick_count::tick_count;

/// With the game's clock: whether the race clock's step was due (the original's carry
/// clear).
pub fn step_race_clock_now() -> bool {
    unsafe { step_race_clock(tick_count()) }
}
