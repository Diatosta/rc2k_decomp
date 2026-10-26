//! `menu::enter_record_entry` (0x4A3C7F): the current player (`dword_75598C`) entered as
//! the current record's event's tenth record entry (its +0x57C): +0x14 the time (+0x88),
//! +0x1C its +0x90, +8 its +4, +0xC its name pointer (+0x18), +0x10 its +8, +0x4C its
//! +0xD0, +0x50 `g_cheats`, +0x54 / +0x58 the sealed time (`menu::sealed_player_time` with
//! the event's number), +4 the player + 1, +0x40 1, +0x20 the name's 0x1F bytes; the
//! event's number into `dword_7559EC` first; then moved up (`menu::rank_record_entry`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `menu::record_entries_pass`. The steps are their Rust versions. Refuses a record past
//! the table, no events table, a current player past 3 or missing, or no name, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::cheats_store::CHEAT_VARS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::standings::*;

// hhmm
/// The Rust version (see the header).
pub fn enter_record_entry() -> Result<(), Refused> {
    let event = current_event()?;
    let n: u32 = { EVENT_STATE.current_player.get() };
    if n >= 4 || player(n) == 0 || get(player(n) + 0x18)? == 0 {
        return Err(BadInput("a current player past 3, missing or without a name").into());
    }
    let p = player(n);
    let slot = (event - { EVENTS_RVA.get() }) / 0x618;
    DRAW_WORK.figure.set(slot);
    let cheats = { CHEAT_VARS.cheats.get() };
    let sealed = crate::menu::sealed_player_time::sealed_player_time(slot)?;
    // Converted: `rc2k_core::menu::bests::enter_tenth`, then `rank_entries`.
    use rc2k_core::menu::bests;
    let name_at = get(p + 0x18)?;
    let best = bests::NewBest {
        key: get(p + 0x88)?,
        player: n + 1,
        nationality: get(p + 4)?,
        name_at,
        car: get(p + 8)?,
        group: get(p + 0x90)?,
        name: bytes(name_at)?,
        entered: Some(get(p + 0xD0)?),
        seal: Some(bests::Seal { cheats, sealed }),
        line: None,
    };
    bests::enter_tenth(
        &mut crate::menu::rank_record_entry::event_list(event)?,
        &best,
    );
    crate::menu::rank_record_entry::rank_entries(event)?;
    Ok(())
}
