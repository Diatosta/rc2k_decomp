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
use crate::framework::rva;
use crate::gfx::block::{game_addr, game_read, game_write};
use crate::gfx::build_glyphs::{BUILDER_FLAGS_RVA, TABLE_RVA, build_glyphs};
use crate::gfx::glyph_builder::RUN_LENGTHS;
use crate::gfx::glyph_tables::{
    TABLES_LIST_RVA, begin_glyph_table, begin_glyph_tables, end_glyph_tables,
};
use crate::memory::MemoryList;

// ---- game data ----

const SPRITE_TABLE_RVA: usize = rva(0x755581); // g_spriteTable

// ---- Rust version ----

/// The table's address goes to `sprite_table`.
pub unsafe fn build_menu_sprites(sprite_table: *mut u32) -> Result<(), Refused> {
    unsafe {
        let list = game_addr(TABLES_LIST_RVA) as *mut MemoryList;
        begin_glyph_tables()?;
        game_write(BUILDER_FLAGS_RVA, RUN_LENGTHS);
        begin_glyph_table(list)?;
        build_glyphs(
            crate::gfx::texts::address(&crate::gfx::texts::MBUTS) as *const u8,
            list,
        )?;
        sprite_table.write_unaligned(game_read(TABLE_RVA));
        end_glyph_tables()?;
        Ok(())
    }
}

/// With the game's `g_spriteTable`.
pub unsafe fn build_menu_sprites_in_game() -> Result<(), Refused> {
    unsafe { build_menu_sprites(game_addr(SPRITE_TABLE_RVA) as *mut u32) }
}
