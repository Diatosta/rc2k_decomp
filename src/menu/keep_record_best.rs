//! `menu::keep_record_best` (0x4A3E8B): the current record (`g_recordTable`, `section`,
//! `index`) takes each counted player's (`dword_755984`) score (+0x94) while it beats the
//! record's (+0x30, signed), with the player's +4 / +8 (+0x74 / +0x78) and name (0x20
//! bytes to +0x54); it stops at the first that does not. `dword_75598C` the player looked
//! at last.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. `records::find_record`'s Rust version. Refuses a record past the
//! table, a player count of 0 or past 4, a missing player or name, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::menus;
use crate::menu::standings::*;
use crate::records::stages_store::{Holder, RecordRef};
use crate::system::owner::system;

/// The current record (`g_recordTable`, `section`, `index`), refused past the table.
fn current_record() -> Result<RecordRef, BadInput> {
    let table = system().records.stage_tables.championship();
    let section: u32 = SECTION_RVA.get();
    let index: u32 = INDEX_RVA.get();
    if section >= 64
        || index >= 64
        || !crate::records::find_record::has_section(table, section, 600)?
    {
        return Err(BadInput("a record past the table"));
    }
    crate::records::find_record::find_record_in(table, index as i32, section)
}
/// The Rust version (see the header): the registers it leaves.
pub fn keep_record_best() -> Result<(), Refused> {
    let screens = &menus().screens;
    let players = counted_players(screens)?;
    if any_unnamed(&players)? {
        return Err(BadInput("a player without a name").into());
    }
    let record = current_record()?;
    // Converted: `rc2k_core::menu::bests::takes_record`.
    use rc2k_core::menu::bests;
    let record = record.record()?;
    for (n, &p) in players.iter().enumerate() {
        menus().screens.event.current_player.set(n as u32);
        let p = p.get();
        if !bests::takes_record(p.figure, record.holder(Holder::Fastest).speed) {
            break;
        }
        let (row, name) = as_holder(&p)?;
        record.set_holder(Holder::Fastest, &row, &name);
    }
    Ok(())
}
