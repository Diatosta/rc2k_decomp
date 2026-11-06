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
use crate::gfx::build_glyphs::{SheetVars, build_glyphs};
use crate::gfx::decoder::DECODER;
use crate::gfx::glyph_builder::RUN_LENGTHS;
use crate::gfx::glyph_tables::{TABLES, begin_glyph_table, begin_glyph_tables, end_glyph_tables};

// ---- game data ----

// ---- Rust version ----

/// The table's address goes to `sprite_table`.
pub fn build_menu_sprites(
    vars: &SheetVars,
    sprite_table: &crate::session::Dword,
) -> Result<(), Refused> {
    let list = crate::pointers::GamePtr::to(&TABLES.memory);
    begin_glyph_tables(vars)?;
    DECODER.builder_flags.set(RUN_LENGTHS);
    begin_glyph_table(vars, list)?;
    build_glyphs(vars, &crate::gfx::texts::MBUTS, list)?;
    sprite_table.set(vars.table.get());
    end_glyph_tables()?;
    all_direct_colour(vars.table.get())
}

/// Refuses a sprite table with a record whose pixels go through a palette: the sprites are
/// drawn with none (`gfx::blit_glyph::BlitCall::sprite`; the original hands them leftover
/// values it never reads).
fn all_direct_colour(table: u32) -> Result<(), Refused> {
    use rc2k_core::strings::bytes::Bytes as _;
    let surface = crate::gfx::surface::surface();
    let records = crate::gfx::glyph_store::bytes(table, "the menus' sprite table")?;
    let count = records.u32_le(0).unwrap_or(0) / 4;
    for glyph in 0..count {
        let record = crate::gfx::blit_glyph::glyph_record(&surface, table, glyph)?;
        if !rc2k_core::gfx::blit::direct_colour(record.u32_le(0).unwrap_or(0)) {
            return Err(
                crate::errors::BadInput("a menu sprite whose pixels need a palette").into(),
            );
        }
    }
    Ok(())
}

/// With the game's `g_spriteTable`.
pub fn build_menu_sprites_in_game() -> Result<(), Refused> {
    build_menu_sprites(
        &crate::gfx::owner::gfx().sheet,
        &crate::menu::owner::menus().screens.loop_flags.sprite_table,
    )
}
