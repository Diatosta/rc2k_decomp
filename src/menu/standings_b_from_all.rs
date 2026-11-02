//! `menu::standings_b_from_all` (0x4A44A0): standings table B emptied
//! (`menu::init_standings_b`) and filled from the drivers in order: +4 0, +0x10 / +8 /
//! +0xC / +0 / +0x34 the driver's +8 / +0x14 / +0x18 / +0 / +0x10, +0x38 .. +0x6C its
//! +0x5C .. +0x90, +0x14 the text of its name (`g_driverNames`, 0x14 each, by its +0);
//! then each group's (+0x90, 0 .. 3) counted players into the entries from 89, 119, 169
//! and 179 down (`menu::player_to_standing_b`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers, `sub_4A9583`. The steps are their Rust versions
//! (`strings::append_string_by_id`'s for the names). Refuses no table, more drivers than its 200
//! entries, a name past the entry, a player count of 0 or past 4, a player missing or without a
//! name, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::standings::*;

/// Each group's (0 .. 3) players, in order, into the table's entries from the group's slot
/// down (`player_to_standing_b`, ebx the player's index).
fn players_by_group(firsts: [usize; 4]) -> Result<(), Refused> {
    let players = counted_players()?;
    let groups: Vec<u32> = players.iter().map(|p| p.get().word(0x90)).collect();
    // Converted: `rc2k_core::menu::standings::by_group`.
    for (n, e) in rc2k_core::menu::standings::by_group(&groups, firsts) {
        { crate::menu::player_to_standing_b::player_to_standing_b(e, players[n], n as u32) }?;
    }
    Ok(())
}

/// The Rust version (see the header): the registers it leaves.
pub fn standings_b_from_all() -> Result<(), Refused> {
    let drivers = walked_drivers()?;
    if drivers.len() > 200 {
        return Err(BadInput("more drivers than the standings table holds").into());
    }
    let players = counted_players()?;
    if any_unnamed(&players)? {
        return Err(BadInput("a player without a name").into());
    }
    let mut texts = Vec::new();
    for d in &drivers {
        let id: u32 = crate::menu::drivers_store::name_string(d.number);
        let mut text = Vec::new();
        crate::strings::append_string_by_id::push_string(&mut text, id)?;
        text.push(0);
        if text.len() > rc2k_core::saves::rankings::HISTORY_LEN - 0x14 {
            return Err(BadInput("a driver's name past its entry").into());
        }
        texts.push(text);
    }
    crate::menu::init_standings_b::init_standings_b()?;
    // Converted: `rc2k_core::saves::rankings::fill_b_from_driver`.
    with_b(|t| {
        for ((d, text), e) in drivers.iter().zip(&texts).zip(&mut t.entries) {
            rc2k_core::saves::rankings::fill_b_from_driver(e, d, text)
                .expect("the name's length checked");
        }
    })?;
    use rc2k_core::menu::standings::B_GROUP_FIRSTS;
    players_by_group(B_GROUP_FIRSTS)?;
    Ok(())
}
