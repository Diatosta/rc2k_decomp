//! `hud::add_solid_quad` (0x4B7216): the quad record's texture places all on one texel
//! (0xF8 << 16, 8 << 16: a solid colour), then `hud::add_quad`.
//!
//! In:  nothing (the record's corners and colour set by the caller)
//! Out: as `add_quad` leaves them
//!
//! Callers: the HUD's drawers (code at 0x4B27E1, 0x4B3BF8, 0x4B5AD0, ...). The quad through
//! `part` (1), in the game its Rust version. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose call goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_write};
use crate::hud::add_quad::QUAD_RVA;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::quads::SOLID_PLACES`
pub unsafe fn add_solid_quad(entry: &Regs) -> Result<Regs, Refused> {
    for (n, v) in rc2k_core::hud::quads::SOLID_PLACES {
        unsafe { game_write(QUAD_RVA + 4 * n, v) };
    }
    let mut g = *entry;
    crate::hud::add_triangle::game_part_quad(1, &mut g)?;
    Ok(g)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { add_solid_quad(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
