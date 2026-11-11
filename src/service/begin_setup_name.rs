//! `service::begin_setup_name` (0x46EE81): starts typing the name of the highlighted setup
//! slot: the name is copied to the text entry's backup and cleared, the entry's length is 0
//! and `typing` (an entry is going on) is 1.
//!
//! The action of the slots on the "save setup" menu (six menu tables point at it); from then on the
//! screen loop runs `edit_setup_name`. Refuses a slot past 99.

use crate::errors::BadInput;
use crate::menu::backup_text::backup_text;
use crate::menu::clear_text::clear_text;
use crate::menu::owner::Menus;
use crate::menu::text_entry::set_length;
use crate::service::load_setup_slot::highlighted_slot;
use crate::service::read_setup_slot::{SLOT_COUNT, slot_name};

// ---- Rust version ----

pub fn begin_setup_name(menus: &Menus) -> Result<(), BadInput> {
    let slot = highlighted_slot(menus);
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table"));
    }
    {
        backup_text(menus, slot_name(slot))?;
        clear_text(menus, slot_name(slot))?;
        menus.screens.option_work.typing.set(1u32);
    }
    set_length(menus, 0);
    Ok(())
}
