//! `menu::clear_record_entries`: the current record's event (the stage best times,
//! 0x618 an event, by the record's +0x3C) has its ten entries' (0x9C apart) +0x40 0.
//!
//! Refuses a record past the table or
//! no events table, before anything.
use crate::errors::Refused;
use crate::menu::owner::Screens;
use crate::menu::standings::*;

pub fn clear_record_entries(screens: &Screens, system: &System) -> Result<(), Refused> {
    let event = current_event(screens, system)?;
    with_event(event, rc2k_core::saves::entries::clear_marks)?;
    Ok(())
}
