//! Sends this machine's record to the others as its player data; unless a race is on, lists
//! the players afresh, reads each one's record into its block, sorts the blocks by DirectPlay id
//! and finds this machine's player among them. Returns true when found; false for no object, a
//! failed listing or a missing player. More than eight players (possible only during a race) are
//! refused.

use crate::errors::{BadInput, Refused};
use crate::net::owner::Net;
use crate::net::sort_players::{bubble_blocks, swap_blocks};
use rc2k_core::net::players;
pub(super) fn list_players(net: &Net) -> Result<bool, Refused> {
    let (vars, link) = (&net.vars, &net.link);
    if !link.has_dplay() {
        return Ok(false);
    }
    crate::net::dp_objects::set_player_data(link.dplay(), link.player_id(), &vars.own.bytes());
    if net.lobby().racing.get() != 1 {
        net.race_slot.set_players(0);
        let result = crate::net::dp_objects::enum_players(
            link.dplay(),
            &|id, _kind, short, _flags, _context| {
                crate::net::player_found::callback(net, id, short)
            },
            0,
        );
        if result != 0 {
            return Ok(false);
        }
    }
    let players = net.race_slot.players();
    if players > players::MOST {
        return Err(BadInput("more than eight players").into());
    }
    for n in 0..players {
        net.lobby().data_size.set(0x10);
        let player = vars.player_block(n)?;
        let mut data = [0u8; 0x10];
        let (result, size) =
            crate::net::dp_objects::get_player_data(link.dplay(), player.id.get(), &mut data);
        net.lobby().data_size.set(size);
        if result == 0 {
            player.record.set_bytes(&data[..size as usize]);
        }
    }
    // The logic is in `rc2k_core::net::players` (`bubble` by the ids, `find_own`).
    let id = |n: u32| vars.player_block_or_end(n).id.get();
    bubble_blocks(
        players,
        |k| id(k) > id(k + 1),
        |k| swap_blocks(vars.player_block(k)?, vars.player_block(k + 1)?),
    )?;
    match players::find_own(link.player_id(), net.race_slot.players(), |n| {
        vars.player_block_or_end(n).id.get()
    }) {
        Ok(n) => {
            net.race_slot.set_player(n);
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

/// `net::list_players` with its result dropped, so a failed listing is never reported to the
/// caller (as the game does).
pub(super) fn refresh_players(net: &Net) -> Result<(), Refused> {
    list_players(net)?;
    Ok(())
}
