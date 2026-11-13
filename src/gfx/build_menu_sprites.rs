//! Builds the menus' sprite table: `begin_glyph_tables`, the builder's flags = 1 (run-length
//! coded), a new table on the fonts' list `glyph_tables.memory`, `build_glyphs` on
//! `files\g\graphics\sprites\mbuts00.pcx` (that sheet and `mbuts01.pcx`), the table to the sprite
//! table variable, `end_glyph_tables`. The list is not rewound: every call takes new memory after
//! what is there.

use crate::errors::Refused;
use crate::gfx::build_glyphs::{SheetVars, build_glyphs};
use crate::gfx::glyph_builder::RUN_LENGTHS;
use crate::gfx::glyph_tables::{begin_glyph_table, begin_glyph_tables, end_glyph_tables};

/// The table's address goes to `sprite_table`.
pub fn build_menu_sprites(
    vars: &SheetVars,
    sprite_table: &crate::session::PlainDword,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
    strings: &crate::strings::owner::Strings,
) -> Result<(), Refused> {
    let list = &gfx.fonts.glyph_tables.memory;
    begin_glyph_tables(vars, gfx, system, strings)?;
    gfx.pictures.decoder.builder_flags.set(RUN_LENGTHS);
    begin_glyph_table(vars, list)?;
    build_glyphs(vars, &crate::gfx::texts::MBUTS, list, gfx, system, strings)?;
    sprite_table.set(vars.table.get());
    end_glyph_tables(strings)?;
    all_direct_colour(vars.table.get(), gfx)
}

/// Refuses a sprite table with a record whose pixels go through a palette: the sprites are drawn
/// with none (`gfx::blit_glyph::BlitCall::sprite`).
fn all_direct_colour(table: u32, gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    use rc2k_core::strings::bytes::Bytes as _;
    let surface = crate::gfx::surface::surface(gfx);
    let records = crate::gfx::glyph_store::bytes(&gfx.fonts, table, "the menus' sprite table")?;
    let count = records.u32_le(0).unwrap_or(0) / 4;
    for glyph in 0..count {
        let record = crate::gfx::blit_glyph::glyph_record(&gfx.fonts, &surface, table, glyph)?;
        if !rc2k_core::gfx::blit::direct_colour(record.u32_le(0).unwrap_or(0)) {
            return Err(
                crate::errors::BadInput("a menu sprite whose pixels need a palette").into(),
            );
        }
    }
    Ok(())
}

/// With the game's sprite table variable.
pub fn build_menu_sprites_in_game(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
    menus: &crate::menu::owner::Menus,
    strings: &crate::strings::owner::Strings,
) -> Result<(), Refused> {
    build_menu_sprites(
        &gfx.sheet,
        &menus.screens.loop_flags.sprite_table,
        gfx,
        system,
        strings,
    )
}
