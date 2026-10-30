//! `gfx::build_menu_sprites` (0x47C4A1): builds the menus' sprite table.
//!
//! In:  nothing
//! Out: every register preserved
//! `begin_glyph_tables`, the builder's flags = 1 (run-length coded), a new table on the
//! fonts' list `stru_51B984`, `build_glyphs` on `files\g\graphics\sprites\mbuts00.pcx`
//! (that sheet and `mbuts01.pcx`), the table's address to `g_spriteTable` (0x755581),
//! `end_glyph_tables`. The list is not rewound: every call takes new memory after what
//! is there. Callers: `sub_47C63F` (the main menu's start), `sub_4154CD`, `sub_4703C2`.
//! The registers above are the original's, for reference; a call by address goes through
//! `routes.rs`.

use crate::errors::Refused;
use crate::gfx::build_glyphs::{SHEET, build_glyphs};
use crate::gfx::decoder::DECODER;
use crate::gfx::glyph_builder::RUN_LENGTHS;
use crate::gfx::glyph_tables::{TABLES, begin_glyph_table, begin_glyph_tables, end_glyph_tables};
use crate::menu::loop_store::LOOP_FLAGS;

// ---- game data ----

// ---- Rust version ----

/// The table's address goes to `sprite_table`.
pub fn build_menu_sprites(sprite_table: &crate::session::Dword) -> Result<(), Refused> {
    let list = crate::pointers::GamePtr::to(&TABLES.memory);
    begin_glyph_tables()?;
    DECODER.builder_flags.set(RUN_LENGTHS);
    begin_glyph_table(list)?;
    build_glyphs(
        crate::pointers::GamePtr::to(&crate::gfx::texts::MBUTS),
        list,
    )?;
    sprite_table.set(SHEET.table.get());
    end_glyph_tables()?;
    Ok(())
}

/// With the game's `g_spriteTable`.
pub fn build_menu_sprites_in_game() -> Result<(), Refused> {
    build_menu_sprites(&LOOP_FLAGS.sprite_table)
}
