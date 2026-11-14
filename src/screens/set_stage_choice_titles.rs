//! The stage choice's two titles and its heading text, for the stage chosen: item 0's string is the
//! group's title; the table's title goes to what should be item 1, but its place is worked out from
//! the menu's first dword (its labels) where item 0's is from the menu's address, so it lands in
//! the labels (kept as is); the heading "<string 0x482> n/6: name" is built in the stage heading
//! buffer. Refuses a choice without its tables and what the heading refuses.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Tables;
use crate::screens::draw_stage_records::selection_in;
use crate::screens::stage_panel::{build_second_heading, group_title};
use rc2k_core::menu::items::{ITEM_LEN, MENU_HEAD_LEN};

/// Where "item 1"'s string goes: worked out from the menu's labels' address, as if it were the
/// menu's (kept as is), it lands in the memory after them.
fn item_1_place(tables: &Tables) -> Result<Option<usize>, BadInput> {
    crate::menu::menus_store::labels_slot(
        tables.menus.select_stage.id(),
        MENU_HEAD_LEN + ITEM_LEN + 0x14,
    )
}

pub fn set_stage_choice_titles(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let menus = o.menus;
    let tables = &menus.tables;
    let s = selection_in(menus, system)?.ok_or(BadInput(
        "no stage tables for the choice: the original reads through null",
    ))?;
    let Some(item_1) = item_1_place(tables)? else {
        return Err(BadInput("the menu's first dword is no pointer").into());
    };
    build_second_heading(
        crate::strings::append_string_by_id::StringOwners::of(o),
        s.record,
    )?;
    let group_title = group_title(&menus.play.results, s.group)?;
    tables.menus.select_stage.id().items(&tables.menus)[0]
        .string
        .set(group_title);
    tables
        .menus
        .labels
        .set_dword(item_1, s.table.title(&system.records().stage_tables)?)?;

    Ok(())
}
