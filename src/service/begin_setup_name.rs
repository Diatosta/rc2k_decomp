//! Starts typing the name of the highlighted setup slot: the name is copied to the text entry's
//! backup and cleared, the entry's length is 0 and `typing` (an entry is going on) is 1.
//!
//! The action of the slots on the "save setup" menu; from then on the screen loop runs
//! `edit_setup_name`. Refuses a slot past 99.

use crate::errors::BadInput;
use crate::menu::backup_text::backup_text;
use crate::menu::clear_text::clear_text;
use crate::menu::owner::Menus;
use crate::menu::text_entry::set_length;
use crate::service::load_setup_slot::highlighted_slot;
use crate::service::owner::Service;
use crate::service::read_setup_slot::{SLOT_COUNT, slot_name};

pub fn begin_setup_name(menus: &Menus, service: &'static Service) -> Result<(), BadInput> {
    let slot = highlighted_slot(&menus.tables.pages, &menus.screens.loop_flags);
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table"));
    }
    {
        backup_text(menus, slot_name(service, slot))?;
        clear_text(menus, slot_name(service, slot))?;
        menus.screens.entry.typing.set(true);
    }
    set_length(menus, 0);
    Ok(())
}
