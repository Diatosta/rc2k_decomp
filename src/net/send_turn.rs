//! `net::send_turn` (0x46415B): the own car's messages by its step count (the current
//! car's block +8, `unk_719FE0`): every fourth step (& 3 = 0) `net::send_inputs`, at & 0x1F
//! = 2 `net::send_position`, at & 0xFF = 1 `net::send_extra`.
//!
//! In:  nothing
//! Out: eax the count & 0xFF, or as `send_extra` leaves the registers; the other
//!      registers as the last send made leaves them
//!
//! See `net/mod.rs`. One caller, `net::race_step`. Its calls through `part`. Shadow mode
//! runs the original alone. Self-test: a copy whose count is the test's and whose calls go
//! to noting stand-ins, against the Rust version, on counts that make each send.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::{CAR, Part, SEND_EXTRA, SEND_INPUTS, SEND_POSITION, a, net_entry};
use rc2k_core::net::race::SENT;

/// The Rust version, the count read at `steps`.
pub fn send_turn_with(r: &mut Regs, steps: u32, part: Part) -> Result<(), Refused> {
    let count = || unsafe { (steps as *const u32).read_volatile() };
    // Converted: `rc2k_core::net::race::Kind::due`.
    for (kind, place) in SENT
        .into_iter()
        .zip([SEND_INPUTS, SEND_POSITION, SEND_EXTRA])
    {
        let (due, masked) = kind.due(count());
        r.eax = masked;
        if due {
            part(place, r)?;
        }
    }
    Ok(())
}

/// The Rust version.
pub fn send_turn(r: &mut Regs, part: Part) -> Result<(), Refused> {
    send_turn_with(r, a(CAR) + 8, part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, send_turn)
}
