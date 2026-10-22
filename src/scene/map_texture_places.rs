//! `scene::map_texture_places` (0x42570D): for each texture's place, the slot that
//! shows it.
//!
//! In:  nothing
//! Out: eax = what was last looked at, ecx = 0xFFFFFFFF, edi = the end of the table
//!
//! The 0xFA0 dwords of `dword_5D3AE8` = -1; then for the slots from 0x3FF down to 0:
//! the texture the slot wants (`dword_5DA968`), if under 0x400, has a place
//! (`dword_5D8968`), and that place, if under 0xFA0, gets the slot's number. Of two
//! slots with one place the lower stays.
//! Six callers.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDI, Regs, game_addr};
use crate::scene::textures::{PLACE_SLOT_RVA, PLACES, SLOTS, TEXTURE_PLACE_RVA, WANTED_RVA, slice};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

pub unsafe fn map_texture_places() -> Left {
    // Converted: `rc2k_core::scene::textures::map_places`.
    unsafe {
        let eax = rc2k_core::scene::textures::map_places(
            slice(WANTED_RVA, SLOTS),
            slice(TEXTURE_PLACE_RVA, SLOTS),
            slice(PLACE_SLOT_RVA, PLACES),
        );
        Left::nothing()
            .with(EAX, eax)
            .with(ECX, u32::MAX)
            .with(EDI, game_addr(PLACE_SLOT_RVA) as u32 + PLACES * 4)
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { map_texture_places() })
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
