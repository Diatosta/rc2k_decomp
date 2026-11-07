//! `menu::standings_a_from_all` (0x4A2B75): standings table A filled from the drivers
//! (`menu::standings_from_drivers`: emptied, each driver with its name), then each group's
//! (+0x90, 0 .. 3) counted players into the entries from 89, 139, 169 and 179 down
//! (`menu::copy_standing_entry`).
//!
//! Callers: the event drivers. The steps are their Rust versions. Refuses a player
//! count of 0 or past 4, a player missing or without a name, and what the drivers' fill refuses,
//! before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::standings::*;

/// Each group's (0 .. 3) players, in order, into the table's entries from the group's slot
/// down (`copy_standing_entry`, ebx the player's index).
fn players_by_group(menus: &Menus, screens: &Screens, firsts: [usize; 4]) -> Result<(), Refused> {
    let players = counted_players(screens)?;
    let groups: Vec<u32> = players.iter().map(|p| p.get().class).collect();
    // Converted: `rc2k_core::menu::standings::by_group`.
    for (n, e) in rc2k_core::menu::standings::by_group(&groups, firsts) {
        { crate::menu::copy_standing_entry::copy_standing_entry(menus, e, players[n], n as u32) }?;
    }
    Ok(())
}

/// The Rust version.
pub fn standings_a_from_all(menus: &Menus) -> Result<(), Refused> {
    let screens = &menus.screens;
    let players = counted_players(screens)?;
    if any_unnamed(&players)? {
        return Err(BadInput("a player without a name").into());
    }
    crate::menu::standings_from_drivers::standings_from_drivers(menus)?;
    use rc2k_core::menu::standings::A_GROUP_FIRSTS;
    players_by_group(menus, screens, A_GROUP_FIRSTS)?;
    Ok(())
}
