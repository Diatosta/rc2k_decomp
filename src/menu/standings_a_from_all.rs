//! `menu::standings_a_from_all`: standings table A filled from the drivers
//! (`menu::standings_from_drivers`: emptied, each driver with its name), then each group's
//! (+0x90, 0 .. 3) counted players into the entries from 89, 139, 169 and 179 down
//! (`menu::copy_standing_entry`).
//!
//! Callers: the event drivers. Refuses a player
//! count of 0 or past 4, a player missing or without a name, and what the drivers' fill refuses,
//! before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::{Cars, Play, Screens};
use crate::menu::standings::*;

/// Each group's (0 .. 3) players, in order, into the table's entries from the group's slot
/// down (`copy_standing_entry`).
fn players_by_group(
    screens: &Screens,
    play: &Play,
    firsts: [usize; 4],
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let players = counted_players(screens, play)?;
    let groups: Vec<u32> = players.iter().map(|p| p.get().class).collect();
    for (n, e) in rc2k_core::menu::standings::by_group(&groups, firsts) {
        {
            crate::menu::copy_standing_entry::copy_standing_entry(
                play, e, players[n], n as u32, tables,
            )
        }?;
    }
    Ok(())
}

pub fn standings_a_from_all(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    cars: &Cars,
    screens: &Screens,
    play: &Play,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let players = counted_players(screens, play)?;
    if any_unnamed(&players)? {
        return Err(BadInput("a player without a name").into());
    }
    crate::menu::standings_from_drivers::standings_from_drivers(so, cars, tables)?;
    use rc2k_core::menu::standings::A_GROUP_FIRSTS;
    players_by_group(screens, play, A_GROUP_FIRSTS, tables)?;
    Ok(())
}
