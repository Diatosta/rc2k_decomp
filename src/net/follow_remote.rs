//! `net::follow_remote` (0x464883): a remote car's step in a network race: while it has
//! steps of a glide left (the current car's block +0xFCC, set to 32 by
//! `net::apply_message`), one less, and a 32nd of the way (+0xFC0 .. +0xFC8) added to its
//! place (+0x134 .. +0x13C).
//!
//! In:  nothing
//! Out: eax +0xFC8 after a glide step, else kept; the rest kept
//!
//! See `net/mod.rs`. One caller, `net::race_step`. Calls nothing. Shadow mode runs the
//! original alone. Self-test: a copy whose switch and car block are the test's, against
//! the Rust version: the network off, no steps left, steps left.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::{CAR, ON, Part, a, net_entry};
use rc2k_core::net::race;

/// The Rust version on the switch at `on` and the block at `car`.
pub fn follow_remote_with(r: &mut Regs, on: u32, car: u32, _part: Part) -> Result<(), Refused> {
    let rd = |at: u32| unsafe { (at as *const u32).read_volatile() };
    let wr = |at: u32, v: u32| unsafe { (at as *mut u32).write_volatile(v) };
    // Converted: `rc2k_core::net::race::glide` (the block read only with the race on).
    let three = |at: u32| [rd(car + at), rd(car + at + 4), rd(car + at + 8)];
    if rd(on) == 0 {
        return Ok(());
    }
    let steps = rd(car + race::GLIDE_STEPS);
    if steps == 0 {
        return Ok(());
    }
    let step = three(race::WANTED);
    if let Some((left, place)) = race::glide(true, steps, three(race::PLACE), step) {
        wr(car + race::GLIDE_STEPS, left);
        for (k, v) in (0..).zip(place) {
            wr(car + race::PLACE + 4 * k, v);
        }
        r.eax = step[2];
    }
    Ok(())
}

/// The Rust version.
pub fn follow_remote(r: &mut Regs, part: Part) -> Result<(), Refused> {
    follow_remote_with(r, a(ON), a(CAR), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, follow_remote)
}
