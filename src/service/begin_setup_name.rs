//! `service::begin_setup_name` (0x46EE81): starts typing the name of the highlighted setup
//! slot: the name is copied to the text entry's backup and cleared, the entry's length is 0
//! and `dword_7556A8` (an entry is going on) is 1.
//!
//! In:  nothing
//! Out: every register preserved
//! The action of the slots on the "save setup" menu (six menu tables point at it); from then
//! on the screen loop runs `edit_setup_name`. Refuses a slot past 99.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::menu::backup_text::backup_text;
use crate::menu::clear_text::clear_text;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::text_entry::set_length;
use crate::service::load_setup_slot::highlighted_slot;
use crate::service::read_setup_slot::{SLOT_COUNT, slot_name};

// ---- Rust version ----

pub fn begin_setup_name() -> Result<(), BadInput> {
    let slot = highlighted_slot();
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table"));
    }
    {
        backup_text(slot_name(slot) as usize as u32)?;
        clear_text(slot_name(slot) as usize as u32)?;
        OPTION_WORK.typing.set(1u32);
    }
    set_length(0);
    Ok(())
}
