//! `menu::keep_record_best`: the current record (the record table, `section`,
//! `index`) takes each counted player's (`event.player_count`) score (+0x94) while it beats the
//! record's (+0x30, signed), with the player's +4 / +8 (+0x74 / +0x78) and name (0x20
//! bytes to +0x54); it stops at the first that does not. `event.current_player` the player looked
//! at last.
//!
//! Refuses a record past the table, a player count of 0 or past 4, a missing player or name, before
//! anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::standings::*;
use crate::records::stages_store::{Holder, RecordRef};
use crate::system::owner::System;

/// The current record (the record table, `section`, `index`), refused past the table.
fn current_record(menus: &Menus, system: &System) -> Result<RecordRef, BadInput> {
    let table = system.records().stage_tables.championship();
    let section: u32 = menus.screens.event.section.get();
    let index: u32 = menus.screens.event.stage_in_section.get();
    if section >= 64
        || index >= 64
        || !crate::records::find_record::has_section(
            &system.records().stage_tables,
            table,
            section,
            600,
        )?
    {
        return Err(BadInput("a record past the table"));
    }
    crate::records::find_record::find_record_in(
        &system.records().stage_tables,
        table,
        index as i32,
        section,
    )
}

pub fn keep_record_best(menus: &Menus, system: &System) -> Result<(), Refused> {
    let players = counted_players(&menus.screens, &menus.play)?;
    if any_unnamed(&players)? {
        return Err(BadInput("a player without a name").into());
    }
    let record = current_record(menus, system)?;
    use rc2k_core::menu::bests;
    let record = record.record(&system.records().stage_tables)?;
    for (n, &p) in players.iter().enumerate() {
        menus.screens.event.current_player.set(n as u32);
        let p = p.get();
        if !bests::takes_record(p.figure, record.holder(Holder::Fastest).speed) {
            break;
        }
        let (row, name) = as_holder(&menus.play, &p)?;
        record.set_holder(Holder::Fastest, &row, &name);
    }
    Ok(())
}
