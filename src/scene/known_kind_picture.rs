//! `scene::known_kind_picture` (0x41F44D): a known kind of car's first picture.
//!
//! In:  eax = which of the known kinds (held to 0 .. 0x21, signed)
//! Out: eax = the number; everything else preserved
//!
//! `dword_5C5708` at the place: 0x22 dwords, one for each known kind of car.
//! 34 call sites, 33 of them in the front end's set-up.
//! Manual patch with a private copy: a conditional jump in the first bytes.
//! Self-test: 14 places.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, rva};

// ---- game data ----

const TABLE_RVA: usize = rva(0x5C5708); // dword_5C5708

// ---- Rust version ----

pub unsafe fn known_kind_picture(place: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::known_place`.
    let place = rc2k_core::scene::kinds::known_place(place);
    unsafe { game_read(TABLE_RVA + place * 4) }
}
