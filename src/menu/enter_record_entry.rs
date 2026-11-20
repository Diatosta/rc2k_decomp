//! `menu::enter_record_entry`: the current player (`event.current_player`) entered as
//! the current record's event's tenth record entry: its `key` the player's `time`, `group` the
//! player's `class`, `nation` the nationality, `name_id` the name's id, `car` the car number,
//! `entered` the player's `entry`, `cheats` the cheats, `sealed` the sealed time
//! (`menu::sealed_player_time` with the event's number), `player` the player + 1, `mark` 1 and
//! `name` the name's 0x1F bytes; then moved up
//! (`menu::rank_record_entry`).
//!
//! Refuses a record past the table, no events table, a current player past 3 or missing, or no
//! name, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::standings::*;

pub fn enter_record_entry(
    menus: &Menus,
    system: &System,
    race: &crate::race::owner::Race,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let event = current_event(&menus.screens, system, tables)?;
    let n: u32 = menus.screens.event.current_player.get();
    if n >= 4 || player(menus, n).get().name_player().is_none() {
        return Err(BadInput("a current player past 3, missing or without a name").into());
    }
    let p = player(menus, n);
    // The event's place in the table (`current_event`'s).
    let slot = event as u32;
    let cheats = menus.play.cheats.cheats.get();
    let sealed = crate::menu::sealed_player_time::sealed_player_time(menus, race, slot)?;
    // The logic is in `rc2k_core::saves::entries::BestTime::enter`, then `rank_entries`.
    use rc2k_core::menu::bests;
    let name_id = name_id_of(&p.get())?;
    let best = bests::NewBest {
        key: p.get().time,
        player: n + 1,
        nationality: p.get().nationality,
        name_id,
        car: p.get().car_number,
        group: p.get().class,
        name: name_of(&menus.play, &p.get())?,
        entered: Some(p.get().entry),
        seal: Some(bests::Seal { cheats, sealed }),
        line: None,
    };
    with_event(tables, event, |list| list[9].enter(&best))?;
    crate::menu::rank_record_entry::rank_entries(&menus.play, event, tables)?;
    Ok(())
}
