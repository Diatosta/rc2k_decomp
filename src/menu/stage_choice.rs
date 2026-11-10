//! The stage choice's own steps (IDA, 2026-10-15): the UK map with the chosen stage
//! (`menu::draw_stage_map`, 14 callers), and the choice's three settings stepped, each then
//! drawing the map again.

use crate::errors::{FatalError, Refused};
use crate::gfx::owner::gfx;
use crate::menu::owner::Menus;
use crate::screens::draw_stage_records::selection;
use crate::session::Dword;
use crate::system::owner::system;

pub const GROUP_RVA: &Dword = &crate::menu::owner::MENUS.screens.event.stage_group; // dword_7558C8: 0 or 1
pub const SUB_RVA: &Dword = &crate::menu::owner::MENUS.screens.event.stage_sub; // dword_7558D0: 0 .. 5
pub const NUMBER_RVA: &Dword = &crate::menu::owner::MENUS.screens.event.stage_number; // dword_7558CC: 1 .. 6
const MAP_SPRITE: u32 = 0xA0;
/// The map that cannot be loaded: no message.
const NO_MAP: FatalError = FatalError {
    code: 0x8071_2345,
    message: "",
};

/// `menu::draw_stage_map`: the stage by `dword_755700` (2: `off_73555A[dword_7587EC]`'s
/// `dword_7587F0`th; 3: `records::find_record` of `g_recordTable` with `section` and
/// `index`; else the choice's: `off_735512[group][sub]`'s `number`th), its overlay
/// `off_7598AE[+0x3C]`; `ukmap.png` loaded with it (the carry: fatal error 0x80712345),
/// converted, `g_pictureFlags` = `g_screenModeFlags` & 0xFFF, made the surface, sprite 0xA0
/// at (0, 0). Refuses a choice without its tables (the original reads through null).
pub fn draw_stage_map(menus: &Menus) -> Result<(), Refused> {
    let system = system();
    let gfx = gfx();
    use crate::gfx::blit_glyph::blit_glyph_on_surface;
    use crate::gfx::convert_picture::convert_picture_now;
    use crate::gfx::load_picture_with_overlay::load_picture_with_overlay;
    use crate::gfx::picture_as_surface::picture_as_surface;
    let mode: u32 = menus.play.champ_view.stage_mode.get();
    let record = match mode {
        2 => {
            let table: u32 = menus.play.arcade.level.get();
            crate::records::stages_store::row(
                system.records.stage_tables.level_table(table)?,
                menus.play.arcade.stage.get(),
            )?
        }
        3 => crate::records::find_record::find_record_in(
            system.records.stage_tables.championship(),
            menus.screens.event.index.get() as i32,
            menus.screens.event.section.get(),
        )?,
        _ => {
            let s = selection(menus)?.ok_or(crate::errors::BadInput(
                "no stage tables for the choice: the original reads through null",
            ))?;
            crate::records::stages_store::row(s.table, menus.screens.event.stage_number.get())?
        }
    };
    let overlay = crate::menu::texts_store::text_at(crate::session::read_on_in(
        &menus.tables.lists,
        &menus.tables.lists.map_overlays,
        record.record()?.event_slot() as usize,
        "a map overlay",
    )?)?;
    if load_picture_with_overlay(&menus.tables.texts.ukmap_png, &overlay)? {
        return Err(NO_MAP.into());
    }
    convert_picture_now()?;
    gfx.pictures
        .picture
        .flags
        .set(gfx.display.mode.mode_flags.get() & 0xFFF);
    picture_as_surface(gfx)?;
    blit_glyph_on_surface(
        MAP_SPRITE,
        menus.screens.loop_flags.sprite_table.get(),
        0,
        0,
        None,
    )?;
    Ok(())
}

/// How a setting draws the map: the game's drawing, or a counter in the tests.
pub type Draw<'a> = &'a mut dyn FnMut() -> Result<(), Refused>;
/// The choice's settings (converted: `rc2k_core::menu::steps::stage_sub` and
/// `stage_number`): the variable stepped, then the map drawn again (`draw`: the Rust
/// version in the game). When the map refuses, the variable is put back first, so that the
/// original, which then runs, steps it once.
fn step(cell: &Dword, next: impl Fn(u32) -> u32, draw: Draw) -> Result<(), Refused> {
    let old = cell.get();
    cell.set(next(old));
    draw().inspect_err(|_| cell.set(old))
}

/// `menu::switch_stage_group`: the group 0 <-> 1.
pub fn switch_stage_group(draw: Draw) -> Result<(), Refused> {
    step(GROUP_RVA, |g| g ^ 1, draw)
}

/// `menu::next_stage_sub`: 5 or more (unsigned) to 0, else on.
pub fn next_stage_sub(draw: Draw) -> Result<(), Refused> {
    step(
        SUB_RVA,
        |s| rc2k_core::menu::steps::stage_sub(s, true),
        draw,
    )
}

/// `menu::previous_stage_sub`: below 1 (signed) to 5, else back.
pub fn previous_stage_sub(draw: Draw) -> Result<(), Refused> {
    step(
        SUB_RVA,
        |s| rc2k_core::menu::steps::stage_sub(s, false),
        draw,
    )
}

/// `menu::next_stage_number`: 6 or more (unsigned) to 1, else on.
pub fn next_stage_number(draw: Draw) -> Result<(), Refused> {
    step(
        NUMBER_RVA,
        |n| rc2k_core::menu::steps::stage_number(n, true),
        draw,
    )
}

/// `menu::previous_stage_number`: below 2 (signed) to 6, else back one.
pub fn previous_stage_number(draw: Draw) -> Result<(), Refused> {
    step(
        NUMBER_RVA,
        |n| rc2k_core::menu::steps::stage_number(n, false),
        draw,
    )
}
