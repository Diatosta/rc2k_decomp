//! Takes the last character off the name of the highlighted setup slot: a text entry length
//! other than 0 is lowered by one and a NUL is written there.
//!
//! Refuses a slot past the table and a length outside 0 .. 0x1F.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::menu::text_entry::{length, set_length};
use crate::service::load_setup_slot::highlighted_slot;
use crate::service::owner::Service;
use crate::service::read_setup_slot::{NAME_LEN, SLOT_COUNT, slot_name};

/// The name of the highlighted slot, and the text entry's length if it is usable.
pub fn entry(
    menus: &Menus,
    service: &'static Service,
) -> Result<(crate::memory::place::BytePlace, usize), BadInput> {
    let slot = highlighted_slot(&menus.tables.pages, &menus.screens.loop_flags);
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table"));
    }
    let len = length(menus);
    if !(0..NAME_LEN as i32).contains(&len) {
        return Err(BadInput("text length outside the name"));
    }
    Ok((slot_name(service, slot), len as usize))
}

pub(super) fn backspace_setup_name(
    menus: &Menus,
    service: &'static Service,
) -> Result<(), BadInput> {
    let (name, len) = entry(menus, service)?;
    if len != 0 {
        let at = name;
        let mut bytes = at.read(NAME_LEN, "a setup name")?;
        let len = rc2k_core::service::setups::backspace(&mut bytes, len);
        at.write(&bytes, "a setup name")?;
        set_length(menus, len as i32);
    }
    Ok(())
}
