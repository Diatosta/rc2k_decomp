//! `screens::draw_selected_stage_menu` (0x481A78): the stage panel for the stage the game has
//! selected, on a screen whose titles are menu items. Instead of drawing the group's and the
//! table's titles it stores their string ids in items 0 and 1 of the current menu, and it
//! builds the heading text (string 0x482) in its own buffer without drawing it.
//!
//! Reached through the draw lists at 0x758CF6 (screen loop `menu::view_courses_loop`, a menu entry)
//! and 0x758E9E (screen loop `menu::select_stage_loop`, six callers).
//!
//! Screen: confirmed by the owner on 2026-10-10 with `blank on
//! screens::draw_selected_stage_menu`: the stage selection screen in the menu ("Escolha
//! Etapa"), whose two titles sit in menu boxes. Which of the two screen loops that was is not
//! known.
//!
//! The record rows come from one of two best-time tables and are skipped while a table is missing
//! (see `stage_panel::draw_records_for_group`). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::owner::Play;
use crate::screens::draw_stage_records::selection_in;
use crate::screens::stage_panel::{
    build_second_heading, draw_records_for_group, draw_stage_body, group_title, menu_item_id,
};
use crate::text::draw_text::Canvas;

/// The string ids this function gives menu items 0 and 1: the group's and the table's titles.
pub fn menu_titles(
    menus: &Menus,
    system: &crate::system::owner::System,
    play: &Play,
) -> Result<Option<(u32, u32)>, BadInput> {
    let Some(sel) = selection_in(menus, system)? else {
        return Ok(None);
    };
    Ok(Some((
        group_title(&play.results, sel.group)?,
        sel.table.title()?,
    )))
}

/// Returns the carry flag. Refuses a missing selection or menu: the original reads or writes
/// through null.
pub fn draw_selected_stage_menu(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
    race: &crate::race::owner::Race,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let play = &menus.play;
    let screens = &menus.screens;
    let (Some(sel), Some((group_title, table_title))) = (
        selection_in(menus, system)?,
        menu_titles(menus, system, play)?,
    ) else {
        return Err(
            BadInput("no record table is selected: the original reads through null").into(),
        );
    };
    let (Some(item_0), Some(item_1)) = (
        menu_item_id(screens, &menus.tables.menus, 0),
        menu_item_id(screens, &menus.tables.menus, 1),
    ) else {
        return Err(BadInput("there is no current menu: the original writes through null").into());
    };
    item_0.set(group_title);
    race.shared.state.settings.direction.set(sel.group);
    item_1.set(table_title);

    build_second_heading(
        crate::strings::append_string_by_id::StringOwners::of(o),
        race,
        sel.record,
    )?;
    draw_stage_body(o, gfx, sel.record, true, canvas)?;
    draw_records_for_group(o, system, sel.group, sel.record, canvas)
}

//
// The original writes two string ids into whatever menu is current. The test therefore only
// runs where those items already hold exactly these ids, which is on the screen this function
// draws: there the writes change nothing.
