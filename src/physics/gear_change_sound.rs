//! `physics::gear_change_sound` (0x45C551): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`, in edi) told of `g_car` +0x238: at 1 its
//! +0x180 and +0x10 set to 1; at -1 its +0x198 and +0x10 set to 1. A reading: a gear
//! change up or down heard.
//!
//! In:  nothing
//! Out: edi the sound object; the rest kept; no flags read (the caller returns)
//!
//! Caller: `sub_43DC72` (its tail).
//! Refuses (before anything) a null sound object when it would be written (the original
//! writes near address 0).
//! Self-test: a copy of the original (the car number, the object table and the word the
//! test's) on test objects: words 1, -1, 0, 2, cars 0 .. 9.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EDI, Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::gearbox::{GearChange, gear_change};

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust version on the car number, the object table and the word at these places.
// Converted: `rc2k_core::physics::gearbox::gear_change`.
pub unsafe fn gear_change_sound_at(current: u32, table: u32, word: u32) -> Result<Left, Refused> {
    unsafe {
        let object = at(table + 4 * (at(current) & 7));
        if let Some(change) = gear_change(at(word)) {
            if object == 0 {
                return Err(BadInput("a null sound object").into());
            }
            let flag = match change {
                GearChange::Up => 0x180,
                GearChange::Down => 0x198,
            };
            put(object + flag, 1);
            put(object + 0x10, 1);
        }
        Ok(Left::nothing().with(EDI, object))
    }
}

/// The Rust version.
pub unsafe fn gear_change_sound() -> Result<Left, Refused> {
    let g = |a: usize| game_addr(rva(a)) as u32;
    unsafe { gear_change_sound_at(g(0x71B3F8), g(0x512A10), g(0x71A210)) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { gear_change_sound() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
