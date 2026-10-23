//! `net::apply_received` (0x4644A4): the messages received since the last step
//! (`net::copy_received`), each player's in turn (`byte_52AD9C` + player * 0x804C +
//! 0x4040: a count, then records of 0x100 bytes, +1 the length, +2 the message) applied
//! (`net::apply_message`, inside `pusha`: ebx the message, ecx its length, ebp the player).
//! The players' loop ends on `jnz` against `dword_52AD98`: with no players it would not end.
//! The receiving thread keeps a count to 0x40 (`sub_419677`), the room there is.
//!
//! In:  nothing
//! Out: eax kept; ebx, ecx, edx, esi as the last player's loop leaves them; ebp the
//!      players; edi kept
//!
//! See `net/mod.rs`. One caller, `net::race_step`. Its calls through `part`. A call at its
//! start: private copy (0x48 bytes, both calls relocated), manual patch. Refuses (before
//! anything) no players and more than 16. Shadow mode runs the original alone. Self-test:
//! a copy whose records and count are the test's and whose calls go to noting stand-ins,
//! against the Rust version: 1 .. 3 players, 0 .. 3 messages each.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::field;
use crate::framework::Regs;
use crate::net::copy_received::MOST_PLAYERS;
use crate::net::types::NetPlayerBlock;
use crate::net::{APPLY, COPY_RECEIVED, PLAYERS, Part, RECEIVED, RECEIVED_LEN, a, net_entry};

/// The Rust version on the records at `received` and the count at `players`.
pub fn apply_received_with(
    r: &mut Regs,
    received: u32,
    players: u32,
    part: Part,
) -> Result<(), Refused> {
    let count = || unsafe { (players as *const u32).read_volatile() };
    if count() == 0 || count() > MOST_PLAYERS {
        return Err(BadInput("no players, or more than the records' 16").into());
    }
    part(COPY_RECEIVED, r)?;
    r.ebp = 0;
    loop {
        r.esi = received + RECEIVED_LEN * r.ebp + field!(NetPlayerBlock, copied_count);
        r.edx = unsafe { (r.esi as *const u32).read_unaligned() };
        if r.edx != 0 {
            r.esi += 4;
            loop {
                r.ecx = unsafe { ((r.esi + 1) as *const u8).read() } as u32;
                r.ebx = r.esi + 2;
                let mut inner = *r;
                part(APPLY, &mut inner)?;
                r.esi += 0x100;
                r.edx = r.edx.wrapping_sub(1);
                if r.edx == 0 {
                    break;
                }
            }
        }
        r.ebp += 1;
        if r.ebp == count() {
            return Ok(());
        }
    }
}

/// The Rust version.
pub fn apply_received(r: &mut Regs, part: Part) -> Result<(), Refused> {
    apply_received_with(r, a(RECEIVED), a(PLAYERS), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, apply_received)
}
