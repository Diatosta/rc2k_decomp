//! `menu::standings_b_from_all`: standings table B emptied
//! (`menu::init_standings_b`) and filled from the drivers in order: `player` 0, `group` / `nation` / `car` / `first` / `total`
//! the driver's `class` / `nation` / `car` / `name_entry` / `total`, `overall` and `stage` its
//! `results`, `name` the text of its name (the driver names, 0x14 each, by its `name_entry`);
//! then each group's (the class, 0 .. 3) counted players into the entries from 89, 119, 169
//! and 179 down (`menu::player_to_standing_b`).
//!
//! The names come from `strings::append_string_by_id`. Refuses no table, more drivers than its 200
//! entries, a name past the entry, a player count of 0 or past 4, a player missing or without a
//! name, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::standings::*;

/// Each group's (0 .. 3) players, in order, into the table's entries from the group's slot
/// down (`player_to_standing_b`).
fn players_by_group(
    menus: &Menus,
    firsts: [usize; 4],
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let players = counted_players(&menus.screens, &menus.play)?;
    let groups: Vec<u32> = players.iter().map(|p| p.get().class).collect();
    for (n, e) in rc2k_core::menu::standings::by_group(&groups, firsts) {
        {
            crate::menu::player_to_standing_b::player_to_standing_b(
                menus, e, players[n], n as u32, tables,
            )
        }?;
    }
    Ok(())
}

pub fn standings_b_from_all(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let drivers = walked_drivers(&menus.cars)?;
    if drivers.len() > 200 {
        return Err(BadInput("more drivers than the standings table holds").into());
    }
    let players = counted_players(&menus.screens, &menus.play)?;
    if any_unnamed(&players)? {
        return Err(BadInput("a player without a name").into());
    }
    let mut texts = Vec::new();
    for d in &drivers {
        let id: u32 = crate::menu::drivers_store::name_string(&menus.cars, d.number)?;
        let mut text = Vec::new();
        crate::strings::append_string_by_id::push_string(so, &mut text, id)?;
        text.push(0);
        if text.len() > rc2k_core::saves::rankings::HISTORY_LEN - 0x14 {
            return Err(BadInput("a driver's name past its entry").into());
        }
        texts.push(text);
    }
    crate::menu::init_standings_b::init_standings_b(tables)?;
    with_b(tables, |t| {
        for ((d, text), e) in drivers.iter().zip(&texts).zip(&mut t.entries) {
            rc2k_core::saves::rankings::fill_b_from_driver(e, d, text)
                .expect("the name's length checked");
        }
    })?;
    use rc2k_core::menu::standings::B_GROUP_FIRSTS;
    players_by_group(menus, B_GROUP_FIRSTS, tables)?;
    Ok(())
}
