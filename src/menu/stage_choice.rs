//! The stage choice's own steps: the UK map with the chosen stage
//! (`menu::draw_stage_map`, 14 callers), and the choice's three settings stepped, each then
//! drawing the map again.

use crate::errors::{FatalError, Refused};
use crate::menu::lists_store::lists;
use crate::menu::owner::Menus;
use crate::menu::texts_store::texts;
use crate::owners::Owners;
use crate::screens::draw_stage_records::selection_in;
use crate::session::PlainDword;

const MAP_SPRITE: u32 = 0xA0;
/// The map that cannot be loaded: no message.
const NO_MAP: FatalError = FatalError {
    code: 0x8071_2345,
    message: "",
};

/// `menu::draw_stage_map`: the stage by `ChampView.stage_mode` (2: the arcade level's
/// `Arcade.stage`th; 3: `records::find_record` of the record table with `section` and
/// `index`; else the choice's group, sub and `number`th), its overlay
/// `MenuLists.map_overlays[0][+0x3C]`; `ukmap.png` loaded with it (a failed load: fatal error
/// 0x80712345), converted, the picture flags = the screen mode flags & 0xFFF, made the
/// surface, sprite 0xA0 at (0, 0). Refuses a choice without its tables.
pub fn draw_stage_map(o: Owners<'static>) -> Result<(), Refused> {
    let (menus, system, gfx) = (o.menus, o.system, o.gfx);
    use crate::gfx::blit_glyph::blit_glyph_on_surface;
    use crate::gfx::convert_picture::convert_picture_now_in;
    use crate::gfx::load_picture_with_overlay::load_picture_with_overlay_in;
    use crate::gfx::picture_as_surface::picture_as_surface;
    let stages = &system.records().stage_tables;
    let mode: u32 = menus.play.champ_view.stage_mode.get();
    let record = match mode {
        2 => {
            let table: u32 = menus.play.arcade.level.get();
            crate::records::stages_store::row(
                stages,
                stages.level_table(table)?,
                menus.play.arcade.stage.get(),
            )?
        }
        3 => crate::records::find_record::find_record_in(
            stages,
            stages.championship(),
            menus.screens.event.stage_in_section.get() as i32,
            menus.screens.event.section.get(),
        )?,
        _ => {
            let s = selection_in(menus, system)?.ok_or(crate::errors::BadInput(
                "no stage tables for the choice: the original reads through null",
            ))?;
            crate::records::stages_store::row(
                stages,
                s.table,
                menus.screens.event.stage.number.get(),
            )?
        }
    };
    let overlay = crate::menu::texts_store::text_at(
        lists().map_overlay(record.record(stages)?.event_slot() as usize)?,
    )?;
    if load_picture_with_overlay_in(&texts().ukmap_png, &overlay, gfx, system, o.strings)? {
        return Err(NO_MAP.into());
    }
    convert_picture_now_in(gfx, system)?;
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
        o.gfx,
    )?;
    Ok(())
}

/// How a setting draws the map: the game's drawing, or a counter in the tests.
pub type Draw<'a> = &'a mut dyn FnMut() -> Result<(), Refused>;
/// The choice's settings (converted: `rc2k_core::menu::steps::stage_sub` and
/// `stage_number`): the variable stepped, then the map drawn again (`draw`). When the map
/// refuses, the variable is put back first.
fn step(cell: &PlainDword, next: impl Fn(u32) -> u32, draw: Draw) -> Result<(), Refused> {
    let old = cell.get();
    cell.set(next(old));
    draw().inspect_err(|_| cell.set(old))
}

/// `menu::switch_stage_group`: the group 0 <-> 1.
pub fn switch_stage_group(menus: &Menus, draw: Draw) -> Result<(), Refused> {
    step(&menus.screens.event.stage.group, |g| g ^ 1, draw)
}

/// `menu::next_stage_sub`: 5 or more (unsigned) to 0, else on.
pub fn next_stage_sub(menus: &Menus, draw: Draw) -> Result<(), Refused> {
    step(
        &menus.screens.event.stage.sub,
        |s| rc2k_core::menu::steps::stage_sub(s, true),
        draw,
    )
}

/// `menu::previous_stage_sub`: below 1 (signed) to 5, else back.
pub fn previous_stage_sub(menus: &Menus, draw: Draw) -> Result<(), Refused> {
    step(
        &menus.screens.event.stage.sub,
        |s| rc2k_core::menu::steps::stage_sub(s, false),
        draw,
    )
}

/// `menu::next_stage_number`: 6 or more (unsigned) to 1, else on.
pub fn next_stage_number(menus: &Menus, draw: Draw) -> Result<(), Refused> {
    step(
        &menus.screens.event.stage.number,
        |n| rc2k_core::menu::steps::stage_number(n, true),
        draw,
    )
}

/// `menu::previous_stage_number`: below 2 (signed) to 6, else back one.
pub fn previous_stage_number(menus: &Menus, draw: Draw) -> Result<(), Refused> {
    step(
        &menus.screens.event.stage.number,
        |n| rc2k_core::menu::steps::stage_number(n, false),
        draw,
    )
}
