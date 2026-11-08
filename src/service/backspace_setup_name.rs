//! `service::backspace_setup_name` (0x46D63C): takes the last character off the name of the
//! highlighted setup slot: if the text entry's length is not 0, it is lowered by one and a
//! NUL is written there.
//!
//! Refuses a slot past the table and a length outside 0 .. 0x1F (the original writes wherever the
//! length points).

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::menu::text_entry::{length, set_length};
use crate::service::load_setup_slot::highlighted_slot;
use crate::service::read_setup_slot::{NAME_LEN, SLOT_COUNT, slot_name};

// ---- Rust version ----

/// The name of the highlighted slot, and the text entry's length if it is usable.
pub fn entry(menus: &Menus) -> Result<(crate::place::Place, usize), BadInput> {
    let slot = highlighted_slot(menus);
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table"));
    }
    let len = length();
    if !(0..NAME_LEN as i32).contains(&len) {
        return Err(BadInput("text length outside the name"));
    }
    Ok((slot_name(slot), len as usize))
}

pub fn backspace_setup_name(menus: &Menus) -> Result<(), BadInput> {
    // Converted: `rc2k_core::service::setups::backspace`.
    let (name, len) = entry(menus)?;
    if len != 0 {
        let at = name;
        let mut bytes = at.read(NAME_LEN, "a setup name")?;
        let len = rc2k_core::service::setups::backspace(&mut bytes, len);
        at.write(&bytes, "a setup name")?;
        set_length(menus, len as i32);
    }
    Ok(())
}
