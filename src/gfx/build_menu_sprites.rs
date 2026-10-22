//! `gfx::build_menu_sprites` (0x47C4A1): builds the menus' sprite table.
//!
//! In:  nothing
//! Out: every register preserved
//! `begin_glyph_tables`, the builder's flags = 1 (run-length coded), a new table on the
//! fonts' list `stru_51B984`, `build_glyphs` on `files\g\graphics\sprites\mbuts00.pcx`
//! (that sheet and `mbuts01.pcx`), the table's address to `g_spriteTable` (0x755581),
//! `end_glyph_tables`. The list is not rewound: every call takes new memory after what
//! is there. Callers: `sub_47C63F` (the main menu's start), `sub_4154CD`, `sub_4703C2`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::gfx::build_glyphs::{BUILDER_FLAGS_RVA, TABLE_RVA};
use crate::gfx::glyph_builder::RUN_LENGTHS;
use crate::gfx::glyph_tables::{
    BEGIN_TABLE, BEGIN_TABLES, BUILD_GLYPHS, END_TABLES, Game, Parts, TABLES_LIST_RVA,
};

// ---- game data ----

const SPRITE_TABLE_RVA: usize = rva(0x755581); // g_spriteTable
const SHEETS_RVA: usize = rva(0x732C93); // "files\g\graphics\sprites\mbuts00.pcx"

// ---- Rust version ----

/// `entry` are the registers at entry; the table's address goes to `sprite_table`.
pub unsafe fn build_menu_sprites(
    entry: &Regs,
    parts: &mut dyn Parts,
    sprite_table: *mut u32,
) -> Result<(), Refused> {
    unsafe {
        let mut g = *entry;
        g.ebx = game_addr(TABLES_LIST_RVA) as u32;
        parts.call(BEGIN_TABLES, &mut g)?;
        game_write(BUILDER_FLAGS_RVA, RUN_LENGTHS);
        parts.call(BEGIN_TABLE, &mut g)?;
        g.eax = game_addr(SHEETS_RVA) as u32;
        parts.call(BUILD_GLYPHS, &mut g)?;
        g.eax = game_read(TABLE_RVA);
        sprite_table.write_unaligned(g.eax);
        parts.call(END_TABLES, &mut g)?;
        Ok(())
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let sprite_table = game_addr(SPRITE_TABLE_RVA) as *mut u32;
    let entry = *r;
    unsafe { build_menu_sprites(&entry, &mut Game, sprite_table) }?;
    Ok(())
}
