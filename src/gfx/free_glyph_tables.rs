//! `gfx::free_glyph_tables` (0x474D12): `memory::free_list` on `g_glyphTablesMemory`
//! (0x51B984): the fonts' glyph tables given back.
//!
//! In:  nothing
//! Out: ebx = the list; the flags as `free_list` leaves them (carry clear)
//!
//! Caller: `menu::apply_screen_mode` (between `gfx::free_fonts` and `gfx::build_fonts`).
//! Called directly (2026-10-23).

use crate::errors::Refused;
use crate::framework::{Regs, game_ptr, rva};

const LIST_RVA: usize = rva(0x51B984); // g_glyphTablesMemory

/// The Rust version (see the header).
pub fn free_glyph_tables() -> Result<(), Refused> {
    unsafe { crate::memory::free_list::free_list(game_ptr(LIST_RVA)) }?;
    Ok(())
}

/// The Rust version where the typed tables run it (a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), Refused> {
    free_glyph_tables()?;
    r.ebx = crate::framework::game_addr(LIST_RVA) as u32;
    r.set_carry(false);
    Ok(())
}
