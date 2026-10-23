//! `gfx::free_glyph_tables` (0x474D12): `memory::free_list` on `g_glyphTablesMemory`
//! (0x51B984): the fonts' glyph tables given back.
//!
//! In:  nothing
//! Out: ebx = the list; the flags as `free_list` leaves them (carry clear)
//!
//! Caller: `menu::apply_screen_mode` (between `gfx::free_fonts` and `gfx::build_fonts`).
//! The freeing through `part` (in the game `free_list`'s Rust version). Shadow mode runs
//! the original alone (memory is given back). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::menu::parts::{Part, parted_entry};

const LIST_RVA: usize = rva(0x51B984); // g_glyphTablesMemory

/// The Rust version: the registers it leaves.
pub unsafe fn free_glyph_tables(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.ebx = game_addr(LIST_RVA) as u32;
    part(0, &mut r)?;
    Ok(r)
}

/// In the game: `free_list`'s Rust version; its carry is clear.
pub fn game_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::memory::free_list::free_list(r.ebx as *mut crate::memory::MemoryList) }?;
    r.set_carry(false);
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    parted_entry(
        r,
        &|r, part| unsafe { free_glyph_tables(r, part) },
        game_part,
    )
}
