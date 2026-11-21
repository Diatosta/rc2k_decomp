//! `menu::clear_record_entries`: the current record's event (the stage best times,
//! 0x618 an event, by the record's `event_slot`) has its ten entries' `mark` set to 0.
//!
//! Refuses a record past the table or
//! no events table, before anything.
use crate::errors::Refused;
use crate::menu::owner::Screens;
use crate::menu::standings::*;

pub fn clear_record_entries(
    screens: &Screens,
    system: &System,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let event = current_event(screens, system, tables)?;
    with_event(tables, event, rc2k_core::saves::entries::clear_marks)?;
    Ok(())
}
