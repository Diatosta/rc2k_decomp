//! `net::apply_message` (0x4644EC): one message received from player ebp put into that
//! player's car block (`g_carBlocks[ebp]`), after logging the own player, the step, the
//! message's first byte, "rcvd fm plyr" and ebp. Players 8 and up: nothing more.
//! - 'JGSM': the four input dwords into +0xFD0 .. +0xFDC;
//! - 'XGSM': nine dwords into +0x64 .. +0x84;
//! - 'PGSM': the place into +0xFC0 .. +0xFC8 (where the car is to be), the other 37
//!   dwords where `net::send_position` took them from. Then, after the car's first 1000
//!   steps (+8; `js` on +8 - 1000) and when the place received is 10 or more away from the
//!   car's (+0x134 .. +0x13C, whole numbers: `fild` / `fisub`, `fsqrt`, `flt_5C5D54`) the
//!   car is put there; else +0xFC0 .. +0xFC8 become a 32nd of the way (`sar 5`) and
//!   +0xFCC 32, for `net::follow_remote` to glide it;
//! - others: nothing.
//!
//! In:  ebx = the message (its first dword the tag), ebp = the player
//! Out: eax the first byte (players 8 and up), the tag (others), or the last dword moved
//!      (or the last 32nd); ecx past the dwords read; edi the car block; the rest kept
//!
//! See `net/mod.rs`. One caller, `net::apply_received` (inside `pusha`). Its calls through
//! `part` (the log's). The distance with the original's x87 instructions. Shadow mode runs
//! the original alone (it logs). Self-test: a copy whose block table is the test's and
//! whose calls go to noting stand-ins, against the Rust version: each tag and another,
//! players 0, 7, 8, steps either side of 1000, places near and far.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::net::{LOG_INT, LOG_INT_NL, LOG_TEXT, PLAYER, Part, STEP, a, g, net_entry};
use rc2k_core::net::race::{self, Arrival, Kind};

/// g_carBlocks: the eight cars' blocks.
pub const BLOCKS: usize = 0x71B3B8;
const TEXT: usize = 0x4DE0EE; // "rcvd fm plyr"
const NEAR: usize = 0x5C5D54; // flt_5C5D54: 10.0

/// The Rust version on the block table at `blocks`.
pub fn apply_message_with(r: &mut Regs, blocks: u32, part: Part) -> Result<(), Refused> {
    let rd = |at: u32| unsafe { (at as *const u32).read_unaligned() };
    let wr = |at: u32, v: u32| unsafe { (at as *mut u32).write_unaligned(v) };
    r.eax = unsafe { (r.ebx as *const u8).read() } as u32;
    for (place, eax) in [
        (LOG_INT, g(PLAYER)),
        (LOG_INT, g(STEP)),
        (LOG_INT, r.eax),
        (LOG_TEXT, a(TEXT)),
        (LOG_INT_NL, r.ebp),
    ] {
        let mut l = Regs { eax, ..*r };
        part(place, &mut l)?;
    }
    if r.ebp >= 8 {
        return Ok(());
    }
    let block = rd(blocks + 4 * r.ebp);
    r.edi = block;
    r.eax = rd(r.ebx);
    // Converted: `rc2k_core::net::race` (`Kind`, `arrival`; the distance's test in `f64`,
    // exact for the decision).
    let Some(kind) = Kind::of(r.eax) else {
        return Ok(());
    };
    r.ecx = 4;
    for n in 0..kind.fields().len() {
        r.eax = rd(r.ebx + r.ecx);
        wr(block + kind.destination(n), r.eax);
        r.ecx += 4;
    }
    if kind != Kind::Position {
        return Ok(());
    }
    let three = |at: u32| [rd(block + at), rd(block + at + 4), rd(block + at + 8)];
    let (wanted, now) = (three(race::WANTED), three(race::PLACE));
    let steps = rd(block + race::STEPS);
    let near = f32::from_bits(unsafe { game_read(rva(NEAR)) });
    match race::arrival(wanted, now, steps, near) {
        Arrival::Jump => {
            for (k, v) in (0..).zip(wanted) {
                wr(block + race::PLACE + 4 * k, v);
            }
        }
        Arrival::Glide(step) => {
            for (k, v) in (0..).zip(step) {
                r.eax = v;
                wr(block + race::WANTED + 4 * k, v);
            }
            wr(block + race::GLIDE_STEPS, race::GLIDE);
        }
    }
    Ok(())
}

/// The Rust version.
pub fn apply_message(r: &mut Regs, part: Part) -> Result<(), Refused> {
    apply_message_with(r, a(BLOCKS), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, apply_message)
}
