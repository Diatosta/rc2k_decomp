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
use crate::menu::event_store::EVENT_STATE;
use crate::menu::standings::*;

/// The current record (`g_recordTable`, `section`, `index`), refused past the table.
fn current_record() -> Result<u32, BadInput> {
    let table = crate::records::stages_store::STAGE_TABLES.championship();
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
    let players = counted_players()?;
    if any_unnamed(&players)? {
        return Err(BadInput("a player without a name").into());
    }
    let record = current_record()?;
    // Converted: `rc2k_core::menu::bests::{takes_record, take_record}`.
    use rc2k_core::menu::bests;
    let mut r = table(record, 0xA4, 1)?;
    let mut r = r.entries();
    for (n, &p) in players.iter().enumerate() {
        EVENT_STATE.current_player.set(n as u32);
        let figure = get(p + 0x94)?;
        if !bests::takes_record(figure, r.get(0, bests::RECORD_HOLDER.figure)) {
            break;
        }
        let holder = bests::Holder {
            figure,
            nationality: get(p + 4)?,
            car: get(p + 8)?,
            name: bytes(get(p + 0x18)?)?,
        };
        bests::take_record(&mut r, 0, &bests::RECORD_HOLDER, &holder);
    }
    Ok(())
}
