//! This player's race is done: its time goes to the record and its total, its state becomes
//! "raced", "after a race" is set and the players are listed again (the record is sent).

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::store::PlayerRecord;
pub fn race_over(net: &Net, time: u32) -> Result<(), Refused> {
    net.vars.all_raced.set(0);
    net.vars.race_over.set(0);
    net.vars.own.race_time.set(time);
    net.vars
        .own
        .total
        .set(net.vars.own.total.get().wrapping_add(time));
    // The logic is in `rc2k_core::net::players::raced`.
    net.vars.own.head.set(
        PlayerRecord::STATE,
        rc2k_core::net::players::raced(net.vars.own.state()),
    );
    net.lobby().after_race.set(1);
    crate::net::list_players::list_players_regs(net)
}
