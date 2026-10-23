//! `physics::reset_when_stuck` (0x43EFBA): with the grades' mode (`dword_71A234`, `g_car`
//! +0x25C) 0 or 2 the stuck count (+0x1E8, `dword_71A1C0`) is set to 0; else it goes up,
//! and at 700 or more (unsigned) `physics::reset_car` puts the car back on the line.
//!
//! In:  nothing
//! Out: when `reset_car` ran, the registers as it leaves them; else all kept; no flags
//!      read (the caller's next is a call)
//!
//! Caller: `sub_43B635` (0x43B900), every physics step.
//! Refuses what `reset_car` refuses. Shadow mode runs the original alone
//! (`physics/reset_parts.rs`).
//! Self-test: a copy whose call goes to a stand-in (place 15), against the Rust version
//! with the same: modes 0 .. 3 and counts about 700.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::physics::reset_parts::{RESET_CAR, reset_entry};
use crate::race::car_starts::Part;
use rc2k_core::physics::start::stuck_tick;

const MODE: usize = rva(0x71A234); // g_car + 0x25C
const STUCK: usize = rva(0x71A1C0); // g_car + 0x1E8

/// The Rust version, `reset_car` through `part` (place 15).
// Converted: `rc2k_core::physics::start::stuck_tick`.
pub unsafe fn reset_when_stuck(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    unsafe {
        let mode: u32 = game_read(MODE);
        let mut stuck = if mode == 2 || mode == 0 {
            0
        } else {
            game_read::<u32>(STUCK)
        };
        let reset = stuck_tick(mode, &mut stuck);
        game_write(STUCK, stuck);
        if reset {
            part(RESET_CAR, &mut r)?;
        }
    }
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    reset_entry(r, reset_when_stuck)
}
