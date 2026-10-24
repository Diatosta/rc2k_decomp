//! `sound::start_race_clock` (0x40F1E9): sets the clock of the race's sounds going.
//!
//! In:  nothing
//! Out: every register preserved
//! The last turn's time = the clock now, the turns a second = 50.0, the turns counted =
//! 0 (`sound/race_clock.rs`). Two callers, `sub_40D378` and `sub_40D592`.
//! Shadow mode runs the original alone (the clock moves) and checks what it left.
//! Manual patch with a private copy: its second instruction is a call.
//! Self-test: a redirected copy with the test's clock.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::sound::race_clock::start_race_clock;
use crate::system::tick_count::tick_count;

/// With the game's clock.
pub fn start_race_clock_now() {
    unsafe { start_race_clock(tick_count()) };
}
