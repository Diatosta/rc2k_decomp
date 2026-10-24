//! `menu::keep_record_best` (0x4A3E8B): the current record (`g_recordTable`, `section`,
//! `index`) takes each counted player's (`dword_755984`) score (+0x94) while it beats the
//! record's (+0x30, signed), with the player's +4 / +8 (+0x74 / +0x78) and name (0x20
//! bytes to +0x54); it stops at the first that does not. `dword_75598C` the player looked
//! at last.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. `records::find_record`'s Rust version. Refuses a record
//! past the table, a player count of 0 or past 4, a missing player or name, before
//! anything. Self-test: the original and the Rust version on the game's tables, scores by
//! lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::menu::standings::*;

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C

/// The current record (`g_recordTable`, `section`, `index`), refused past the table.
fn current_record() -> Result<u32, BadInput> {
    let table = game_addr(RECORDS_RVA) as u32;
    let section: u32 = unsafe { game_read(SECTION_RVA) };
    let index: u32 = unsafe { game_read(INDEX_RVA) };
    if section >= 64
        || index >= 64
        || !crate::records::find_record::has_section(table, section, 600)
    {
        return Err(BadInput("a record past the table"));
    }
    Ok(crate::records::find_record::find_record_in(
        table,
        index as i32,
        section,
    ))
}
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn keep_record_best() -> Result<(), Refused> {
    let players = counted_players()?;
    if players.iter().any(|&p| get(p + 0x18) == 0) {
        return Err(BadInput("a player without a name").into());
    }
    let record = current_record()?;
    // Converted: `rc2k_core::menu::bests::{takes_record, take_record}`.
    use rc2k_core::menu::bests;
    let mut r = unsafe { table(record, 0xA4, 1) };
    for (n, &p) in players.iter().enumerate() {
        unsafe { game_write(CURRENT_RVA, n as u32) };
        let figure = get(p + 0x94);
        if !bests::takes_record(figure, r.get(0, bests::RECORD_HOLDER.figure)) {
            break;
        }
        let holder = bests::Holder {
            figure,
            nationality: get(p + 4),
            car: get(p + 8),
            name: bytes(get(p + 0x18)),
        };
        bests::take_record(&mut r, 0, &bests::RECORD_HOLDER, &holder);
    }
    Ok(())
}
