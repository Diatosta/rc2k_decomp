//! `sound::play_cue_queues` (0x40ED13): the spoken cues of the two players' cars.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Unless no sound object is in use (`g_raceSoundObjectCount` 0): `sound::play_cue_queue`
//! for the object of car `dword_514F70`, then unless `dword_514F78` is 1 (one player) for
//! that of `dword_514F74`. One caller, `sound::race_turn`.
//! Refuses (before anything) a car past 7 or a null object; what the queues refuse puts
//! back what was written (a sound already sent stays sent).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::sound::store::ObjectsSlot;
use crate::sound::turn_parts::{COUNT, OBJECTS};

/// dword_514F70, dword_514F74: the two players' cars; dword_514F78: 1 with one player.
const FIRST: ObjectsSlot = ObjectsSlot::at(0x514F70);
const SECOND: ObjectsSlot = ObjectsSlot::at(0x514F74);
const ONE_PLAYER: ObjectsSlot = ObjectsSlot::at(0x514F78);

fn g(x: impl crate::store::Ida) -> u32 {
    unsafe { crate::store::ida_read(x) }
}

// ---- Rust version ----

/// The players' cars' cue queues played.
pub fn play_cue_queues() -> Result<(), Refused> {
    if g(COUNT) == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::sound::turn::cue_cars`.
    let cars = rc2k_core::sound::turn::cue_cars(g(FIRST), || g(SECOND), g(ONE_PLAYER));
    let object = |n: u32| g(OBJECTS + 4 * n as usize);
    if cars.iter().any(|&n| n > 7 || object(n) == 0) {
        return Err(BadInput("a player's car past 7 or without a sound object").into());
    }
    for n in cars {
        crate::sound::play_cue_queue::play_cue_queue(object(n))?;
    }
    Ok(())
}
