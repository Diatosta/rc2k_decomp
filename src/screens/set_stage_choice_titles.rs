//! `screens::set_stage_choice_titles` (0x469393): the stage choice's two titles and its
//! heading text, for the stage chosen.
//!
//!
//! The menu at 0x748620 (the stage choice's): item 0's string (+0x14) = the group's title
//! (`results.group_titles`); then the table's title (its +0x38) is stored at `[0x748620] + 4 + 0x5C
//! + 0x14`: the original takes the menu's first dword (a pointer to its labels, `LabelTables.select_stage_labels.label.x`)
//! where item 0 took the menu's address, so "item 1" lands in the labels, at 0x747F98 (section 16);
//! the heading "<string 0x482> n/6: name" built in the stage heading buffer
//! (`stage_panel::build_second_heading`). In two entry lists. Refuses a choice without its tables
//! (the original reads through null), and what the heading refuses. The menus and draw lists run
//! its `entry` (`menu::table_calls`).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::owner::Tables;
use crate::screens::draw_stage_records::selection;
use crate::screens::stage_panel::{build_second_heading, table_dword_at};
use rc2k_core::menu::items::{ITEM_LEN, MENU_HEAD_LEN};

/// Where the original puts "item 1"'s string: worked out from the menu's labels' address, as
/// if it were the menu's (the original's slip, `notes/bugs.md`), it lands in the memory after
/// them.
fn item_1_place(tables: &Tables) -> Result<Option<usize>, BadInput> {
    crate::menu::menus_store::labels_slot(
        tables.menus.select_stage.id(),
        MENU_HEAD_LEN + ITEM_LEN + 0x14,
    )
}

pub fn set_stage_choice_titles(menus: &Menus) -> Result<(), Refused> {
    let tables = &menus.tables;
    let s = selection(menus)?.ok_or(BadInput(
        "no stage tables for the choice: the original reads through null",
    ))?;
    let Some(item_1) = item_1_place(tables)? else {
        return Err(BadInput("the menu's first dword is no pointer").into());
    };
    build_second_heading(s.record)?;
    let group_title = table_dword_at(
        &menus.play.results,
        &menus.play.results.group_titles,
        s.group,
    )?;
    tables.menus.select_stage.id().items()[0]
        .string
        .set(group_title);
    crate::pointers::store_value::<u32>(&tables.menus.labels, item_1, s.table.title()?)?;

    Ok(())
}
