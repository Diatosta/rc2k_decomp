//! `drive::replay_clock_tick` (0x460C20): the replay's clock: `dword_5C2534` + 1, at 100
//! back to 0 with the frame `dword_5C2530` + 1.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::replay_step`.
//! Self-test: the original and the Rust version below, at and past 100.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{g, p};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::drive::clocks::ReplayClock;

/// The Rust version.
// Converted: `rc2k_core::drive::clocks::ReplayClock::tick`.
pub fn replay_clock_tick() {
    let was = ReplayClock {
        frame: g(0x5C2530),
        hundredths: g(0x5C2534),
    };
    let mut c = was;
    c.tick();
    p(0x5C2534, c.hundredths);
    if c.frame != was.frame {
        p(0x5C2530, c.frame);
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    replay_clock_tick();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
