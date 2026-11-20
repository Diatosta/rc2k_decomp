//! One turn of waiting for the race to start: this player's state becomes "waiting" and its
//! time -1; once a second the players are listed; on the host, with every player waiting, the
//! sync with 'P' messages runs and when all have answered `race_over` is 1. Returns `race_over`.

use crate::errors::Refused;
use crate::net::lobby_loop::{all_states, sync};
use crate::net::owner::Net;
use rc2k_core::net::players::State;
pub(super) fn race_wait(
    net: &Net,
    clock: &crate::clock::tick_count::PictureClock,
) -> Result<u32, Refused> {
    let vars = &net.vars;
    vars.own.set_state(vars.own.state().waiting());
    vars.own.race_time.set(u32::MAX);
    let now = crate::clock::tick_count::tick_count(clock);
    if (now
        .wrapping_sub(net.lobby().listed_at.get())
        .wrapping_sub(1000) as i32)
        >= 0
    {
        net.lobby().listed_at.set(now);
        crate::net::list_players::refresh_players(net)?;
    }
    if vars.own.state().is_host() {
        if net.race_slot.players() == 0 {
            return Ok(vars.race_over.get());
        }
        if all_states(&net.vars, State::WAITING, net.race_slot.players()) == State::WAITING
            && sync(net, clock, false)?
        {
            vars.race_over.set(1);
        }
    }
    Ok(vars.race_over.get())
}
