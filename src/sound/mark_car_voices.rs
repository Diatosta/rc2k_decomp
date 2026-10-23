//! `sound::mark_car_voices` (0x40D75A): each car's sound object told whether the car
//! sounds and whose voice it has.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Only while `dword_514F8C` is 1: for each of the `dword_514F7C` objects in use
//! (`g_raceSoundObjects[n]`), +8 = 0; when its car (`g_carBlocks[n]`) has +0x25C not 0,
//! +8 = 1 and +0 = 0, and unless +0x25C is 1, +0 = !n (a negative number: a car of the
//! computer's, by the callers' reading). One caller, `sound::race_turn`.
//! Refuses (before writing) a count of 0 (the original's loop would not end) or over 8,
//! and a null object or car.
//! Shadow mode: both, the objects' two dwords compared.
//! Self-test: the original against the Rust version with the two tables pointing at the
//! test's objects and cars (put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::sound::turn_parts::{COUNT, OBJECTS, turn_entry};

/// dword_514F8C: 1 while the cars' voices are to be marked.
const MARKING: usize = 0x514F8C;
/// g_carBlocks: the cars' blocks.
const CARS: usize = 0x71B3B8;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

// ---- Rust version ----

/// The (object, car) pairs in use, or a refusal.
fn pairs() -> Result<Vec<(u32, u32)>, BadInput> {
    let count = g(COUNT);
    if count == 0 || count > 8 {
        return Err(BadInput("a sound object count of 0 or over 8"));
    }
    let pairs: Vec<(u32, u32)> = (0..count as usize)
        .map(|n| (g(OBJECTS + 4 * n), g(CARS + 4 * n)))
        .collect();
    if pairs.iter().any(|&(o, c)| o == 0 || c == 0) {
        return Err(BadInput("a null sound object or car"));
    }
    Ok(pairs)
}

/// Converted: `rc2k_core::sound::cars::voice_mark`.
pub fn mark_car_voices_in(_: &mut Regs) -> Result<(), Refused> {
    if g(MARKING) != 1 {
        return Ok(());
    }
    for (n, (object, car)) in pairs()?.into_iter().enumerate() {
        unsafe {
            let o = object as *mut u32;
            let kind = ((car + 0x25C) as *const u32).read_unaligned();
            let (sounds, voice) = rc2k_core::sound::cars::voice_mark(n as u32, kind);
            o.add(2).write_unaligned(sounds);
            if let Some(voice) = voice {
                o.write_unaligned(voice);
            }
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, mark_car_voices_in)
}
