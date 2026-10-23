//! `physics::sound_cue_e5` (0x45D028): when the current car's entry in `dword_641680` is
//! not -1, its sound object (`g_raceSoundObjects[g_currentCar & 7]`) gets +0x24C 0xE5,
//! +0x250 0 and +0x10 1 (a reading: a sound to play). All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller returns)
//!
//! Caller: `sub_45EDB5` (its tail).
//! Refuses (before anything) a current car past 7, or a null sound object when written.
//! Self-test: a copy of the original (the car number, both tables the test's) on test
//! objects: cars 0 .. 7, entries -1 and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::listed_cars::{CUE_E5, cue_e5_object};

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The core: the car number, the listed table, the sound objects at these places.
// Converted: `rc2k_core::physics::listed_cars::{cue_e5_object, CUE_E5}`.
pub unsafe fn sound_cue_e5_at(current: u32, listed: u32, objects: u32) -> Result<Left, Refused> {
    unsafe {
        let object = cue_e5_object(
            at(current),
            |car| at(listed + 4 * car),
            |slot| at(objects + 4 * slot),
        )?;
        if let Some(object) = object {
            for (offset, v) in CUE_E5 {
                put(object + offset, v);
            }
        }
        Ok(Left::nothing())
    }
}

/// The Rust version.
pub unsafe fn sound_cue_e5() -> Result<Left, Refused> {
    let g = |a: usize| game_addr(rva(a)) as u32;
    unsafe { sound_cue_e5_at(g(0x71B3F8), g(0x641680), g(0x512A10)) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { sound_cue_e5() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
