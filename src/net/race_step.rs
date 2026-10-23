//! `net::race_step` (0x464133): a car's step in a network race (nothing without one,
//! `dword_52A910` 0): for the own player's car (`g_currentCar` = `dword_52A920`) its
//! messages sent (`net::send_turn`) and the others' applied (`net::apply_received`); for
//! another's, its glide (`net::follow_remote`).
//!
//! In:  nothing
//! Out: eax `g_currentCar` (when on), or as the calls leave the registers
//!
//! See `net/mod.rs`. One caller, `drive::replay_step` (each car's step, after its
//! physics). Its calls through `part`. Shadow mode runs the original alone. Self-test: a
//! copy whose three variables are the test's and whose calls go to noting stand-ins,
//! against the Rust version: off, the own car, another.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::{APPLY_RECEIVED, FOLLOW, ON, PLAYER, Part, SEND_TURN, a, net_entry};

/// g_currentCar.
const CURRENT: usize = 0x71B3F8;

/// Where the variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub on: u32,
    pub current: u32,
    pub player: u32,
}

/// The Rust version on the variables in `v`.
pub fn race_step_with(r: &mut Regs, v: &Vars, part: Part) -> Result<(), Refused> {
    let rd = |at: u32| unsafe { (at as *const u32).read_volatile() };
    if rd(v.on) == 0 {
        return Ok(());
    }
    r.eax = rd(v.current);
    if r.eax == rd(v.player) {
        part(SEND_TURN, r)?;
        part(APPLY_RECEIVED, r)
    } else {
        part(FOLLOW, r)
    }
}

/// The Rust version.
pub fn race_step(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let v = Vars {
        on: a(ON),
        current: a(CURRENT),
        player: a(PLAYER),
    };
    race_step_with(r, &v, part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, race_step)
}
