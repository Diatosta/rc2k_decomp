//! `scene::known_kind_plate` (0x41F519): a known kind of car's kind of number plate.
//!
//! In:  ebx = which of the known kinds (held to 0 .. 0x21, signed)
//! Out: ebx = the number; everything else preserved
//!
//! `dword_5C5A38` at the place: 0x22 dwords, one for each known kind of car.
//! Two call sites in `scene::load_number_plates`.
//! Manual patch with a private copy: a conditional jump in the first bytes.
//! Self-test: 14 places.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, Regs, game_read, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

const TABLE_RVA: usize = rva(0x5C5A38); // dword_5C5A38

// ---- Rust version ----

pub unsafe fn known_kind_plate(place: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::known_place`.
    let place = rc2k_core::scene::kinds::known_place(place);
    unsafe { game_read(TABLE_RVA + place * 4) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EBX, unsafe { known_kind_plate(r.ebx) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
