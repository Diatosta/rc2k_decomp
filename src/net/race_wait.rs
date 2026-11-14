//! One turn of waiting for the race to start: this player's state becomes "waiting" and its
//! time -1; once a second the players are listed; on the host, with every player waiting, the
//! sync with 'P' messages runs and when all have answered `race_over` is 1. Returns `race_over`.

use crate::errors::Refused;
use crate::net::lobby_loop::{all_states, sync};
use crate::net::owner::Net;
use crate::net::store::PlayerRecord;
use rc2k_core::net::players;
pub fn race_wait(net: &Net) -> Result<u32, Refused> {
    let vars = &net.vars;
    // The logic is in `rc2k_core::net::players` (`waiting`, `all_in_state` in `all_states`).
    vars.own
        .head
        .set(PlayerRecord::STATE, players::waiting(vars.own.state()));
    vars.own.race_time.set(u32::MAX);
    let now = crate::system::tick_count::tick_count();
    if (now
        .wrapping_sub(net.edit().clock_a.get())
        .wrapping_sub(1000) as i32)
        >= 0
    {
        net.edit().clock_a.set(now);
        crate::net::list_players::list_players_regs(net)?;
    }
    if vars.own.state() & players::HOST != 0 {
        if vars.players.get() == 0 {
            return Ok(vars.race_over.get());
        }
        if all_states(&net.vars, players::WAITING, vars.players.get()) == players::WAITING
            && sync(net, false)?
        {
            vars.race_over.set(1);
        }
    }
    Ok(vars.race_over.get())
}
