//! `screens::set_stage_choice_titles` (0x469393): the stage choice's two titles and its
//! heading text, for the stage chosen.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The menu at 0x748620 (the stage choice's): item 0's string (+0x14) = the group's title
//! (`dword_758718[group]`); then the table's title (its +0x38) is stored at `[0x748620] + 4 + 0x5C
//! + 0x14`: the original takes the menu's first dword (a pointer to its labels, `stru_747F24`)
//! where item 0 took the menu's address, so "item 1" lands in the labels, at 0x747F98 (section 16);
//! the heading "<string 0x482> n/6: name" built in `byte_A8BD54`
//! (`stage_panel::build_second_heading`). In two entry lists. Refuses a choice without its tables
//! (the original reads through null), and what the heading refuses. The menus and draw lists run
//! its `entry` (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::menus_store::MENUS;
use crate::pointers::GamePtr;
use crate::screens::draw_stage_records::selection;
use crate::screens::draw_time_trial_stage_info::GROUP_TITLES;
use crate::screens::stage_panel::{build_second_heading, table_dword_at};
use rc2k_core::menu::items::{ITEM_LEN, MENU_HEAD_LEN};

/// Where item 0's string goes, and where the original puts "item 1"'s.
fn places() -> Result<(GamePtr, u32), BadInput> {
    // Item 1's place is worked out from the menu's labels' address, as if it were the
    // menu's (the original's slip, `notes/bugs.md`): it lands in the memory after them.
    Ok((
        GamePtr::to(&MENUS.select_stage.items[0].string),
        crate::menu::menus_store::labels_place_of(&MENUS.select_stage.head)?
            .wrapping_add(MENU_HEAD_LEN + ITEM_LEN + 0x14),
    ))
}

pub fn set_stage_choice_titles() -> Result<(), Refused> {
    let s = selection()?.ok_or(BadInput(
        "no stage tables for the choice: the original reads through null",
    ))?;
    let (item_0, item_1) = places()?;
    if item_1 < 0x1_0000 {
        return Err(BadInput("the menu's first dword is no pointer").into());
    }
    build_second_heading(s.record)?;
    item_0.write(
        table_dword_at(
            &crate::menu::results_store::RESULT_FIGURES,
            GROUP_TITLES,
            s.group,
        )?,
        "a menu item's string",
    )?;
    GamePtr::new(item_1).write(s.table.title()?, "a menu item's string")?;

    Ok(())
}
