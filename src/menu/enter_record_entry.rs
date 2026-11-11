//! `menu::enter_record_entry` (0x4A3C7F): the current player (`event.current_player`) entered as
//! the current record's event's tenth record entry (its +0x57C): +0x14 the time (+0x88),
//! +0x1C its +0x90, +8 its +4, +0xC its name pointer (+0x18), +0x10 its +8, +0x4C its
//! +0xD0, +0x50 `g_cheats`, +0x54 / +0x58 the sealed time (`menu::sealed_player_time` with
//! the event's number), +4 the player + 1, +0x40 1, +0x20 the name's 0x1F bytes; the
//! event's number into `draw_work.figure` first; then moved up (`menu::rank_record_entry`).
//!
//! Caller: `menu::record_entries_pass`. Refuses a record past
//! the table, no events table, a current player past 3 or missing, or no name, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::standings::*;

// hhmm
pub fn enter_record_entry(menus: &Menus) -> Result<(), Refused> {
    let event = current_event(&menus.screens)?;
    let n: u32 = menus.screens.event.current_player.get();
    if n >= 4 || player(menus, n).get().name_player().is_none() {
        return Err(BadInput("a current player past 3, missing or without a name").into());
    }
    let p = player(menus, n);
    // The event's place in the table (`current_event`'s).
    let slot = event as u32;
    menus.screens.draw_work.figure.set(slot);
    let cheats = menus.play.cheats.cheats.get();
    let sealed = crate::menu::sealed_player_time::sealed_player_time(menus, slot)?;
    // The logic is in `rc2k_core::saves::entries::BestTime::enter`, then `rank_entries`.
    use rc2k_core::menu::bests;
    let name_at = name_address_of(&p.get())?;
    let best = bests::NewBest {
        key: p.get().time,
        player: n + 1,
        nationality: p.get().nationality,
        name_at,
        car: p.get().car_number,
        group: p.get().class,
        name: name_of(menus, &p.get())?,
        entered: Some(p.get().entry),
        seal: Some(bests::Seal { cheats, sealed }),
        line: None,
    };
    with_event(event, |list| list[9].enter(&best))?;
    crate::menu::rank_record_entry::rank_entries(&menus.play, event)?;
    Ok(())
}
