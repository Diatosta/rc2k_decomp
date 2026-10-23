//! `sound::play_cue_queues` (0x40ED13): the spoken cues of the two players' cars.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Unless no sound object is in use (`g_raceSoundObjectCount` 0): `sound::play_cue_queue`
//! for the object of car `dword_514F70`, then unless `dword_514F78` is 1 (one player) for
//! that of `dword_514F74`. One caller, `sound::race_turn`.
//! Refuses (before anything) a car past 7 or a null object; what the queues refuse puts
//! back what was written (a sound already sent stays sent).
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose two calls go to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::sound::turn_parts::{COUNT, OBJECTS, turn_entry};

/// dword_514F70, dword_514F74: the two players' cars; dword_514F78: 1 with one player.
const FIRST: usize = 0x514F70;
const SECOND: usize = 0x514F74;
const ONE_PLAYER: usize = 0x514F78;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

// ---- Rust version ----

pub fn play_cue_queues_in(r: &mut Regs) -> Result<(), Refused> {
    if g(COUNT) == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::sound::turn::cue_cars`.
    let cars = rc2k_core::sound::turn::cue_cars(g(FIRST), || g(SECOND), g(ONE_PLAYER));
    let object = |n: u32| g(OBJECTS + 4 * n as usize);
    if cars.iter().any(|&n| n > 7 || object(n) == 0) {
        return Err(BadInput("a player's car past 7 or without a sound object").into());
    }
    let mut c = *r;
    for n in cars {
        c.eax = n;
        c.ebx = object(n);
        crate::sound::play_cue_queue::play_cue_queue_in(&mut c)?;
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, play_cue_queues_in)
}
