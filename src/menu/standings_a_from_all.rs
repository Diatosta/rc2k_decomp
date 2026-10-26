//! `menu::standings_a_from_all` (0x4A2B75): standings table A filled from the drivers
//! (`menu::standings_from_drivers`: emptied, each driver with its name), then each group's
//! (+0x90, 0 .. 3) counted players into the entries from 89, 139, 169 and 179 down
//! (`menu::copy_standing_entry`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers, `sub_4A94EC`. The steps are their Rust versions. Refuses a player
//! count of 0 or past 4, a player missing or without a name, and what the drivers' fill refuses,
//! before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::standings::*;

/// Each group's (0 .. 3) players, in order, into the table's entries from the group's slot
/// down (`copy_standing_entry`, ebx the player's index).
fn players_by_group(first: u32, size: u32, firsts: [usize; 4]) -> Result<(), Refused> {
    let players = counted_players()?;
    let groups: Vec<u32> = players
        .iter()
        .map(|&p| get(p + 0x90))
        .collect::<Result<_, _>>()?;
    // Converted: `rc2k_core::menu::standings::by_group`.
    for (n, e) in rc2k_core::menu::standings::by_group(&groups, firsts) {
        {
            crate::menu::copy_standing_entry::copy_standing_entry(
                first + size * e as u32,
                players[n],
                n as u32,
            )
        }?;
    }
    Ok(())
}

/// The Rust version (see the header): the registers it leaves.
pub fn standings_a_from_all() -> Result<(), Refused> {
    let players = counted_players()?;
    if any_unnamed(&players)? {
        return Err(BadInput("a player without a name").into());
    }
    crate::menu::standings_from_drivers::standings_from_drivers()?;
    let first: u32 = TABLE_A_RVA.get();
    use rc2k_core::menu::standings::A_GROUP_FIRSTS;
    players_by_group(first, A_ENTRY, A_GROUP_FIRSTS)?;
    Ok(())
}
