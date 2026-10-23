//! `drive::race_clock_tick` (0x445554): the race's clock in tenths. `scene::restart_clock`
//! (the tick count into `g_sceneClock`), into `dword_61383C`; the last tick kept
//! (`dword_613840`) brought to within 1000 of it (by the sign of the `cmp`); the tenths
//! since (`div` by 10) into `dword_613844` and added back (x 10) to `dword_613840`.
//!
//! In:  nothing
//! Out: eax the tenths x 10, ebx 10, edx the product's high dword (0); the rest kept; no
//!      flags read
//!
//! Caller: `drive::race_step` (`sub_4454D2`). It begins with a call: hooked by a private
//! copy of its 0x55 bytes (the call relocated) and a hand-made patch.
//! Self-test: a copy whose clock call goes to a stand-in giving the test's times, against
//! the Rust version with the same times: within 1000 and beyond, round tenths and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX, Regs};
use crate::physics::flow::{g, p};
use crate::screens::harness::Left;
use rc2k_core::drive::clocks::RaceClock;

/// The Rust version after the clock's reading `now` (already in `g_sceneClock`).
// Converted: `rc2k_core::drive::clocks::RaceClock::tick`.
pub fn tick_from(now: u32) -> Left {
    let mut c = RaceClock {
        counted: g(0x613840),
        ..RaceClock::default()
    };
    let (eax, edx) = c.tick(now);
    p(0x61383C, c.now);
    p(0x613844, c.tenths);
    p(0x613840, c.counted);
    Left::nothing().with(EAX, eax).with(EBX, 10).with(EDX, edx)
}

/// The Rust version.
pub unsafe fn race_clock_tick() -> Left {
    let now = crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
    tick_from(now)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { race_clock_tick() })
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    rust(r)?.apply(r);
    Ok(())
}
