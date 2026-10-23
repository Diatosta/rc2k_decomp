//! `scene::update_carousel_pictures` (0x49029C): the pictures of four carousel
//! entries into the four places that hold pictures.
//!
//! In:  nothing (`dword_75AAA8` .. `dword_75AAB4` the four entries)
//! Out: eax = one past the last texture assigned, ebx = one past the last slot,
//!      ecx = 0, esi = 24, edi = the fourth entry's record
//!
//! `scene/carousel_pictures.rs` has what it does, with the two functions inlined
//! (`scene::keep_held_pictures` 0x49036F, `scene::assign_picture_slots` 0x4903E3).
//! Two callers.
//! Refuses an entry past the records or without one, and slots past the table.
//! Self-test: 60 settings by lot, with records of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read};
use crate::scene::carousel_pictures::{WANTED_PICTURES_RVA, update};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { update(game_read(WANTED_PICTURES_RVA)) }?)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
