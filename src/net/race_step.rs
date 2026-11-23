//! A car's step in a network race (nothing without a session): the own player's car sends its
//! messages and applies the others'; the host's computer drivers' cars send theirs for the others
//! (`net::send_bot_turn`); any other car follows its glide. Called from each car's step after its
//! physics.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::{apply_received, follow_remote, send_turn};
use crate::race::owner::Race;
pub fn race_step(race: &Race, net: &Net) -> Result<(), Refused> {
    // A player who left takes the car out of the race, even when the session is gone.
    crate::net::left::apply(race, net);
    if !net.race_slot.is_on() {
        return Ok(());
    }
    if race.shared.work().current_car == net.race_slot.player() {
        send_turn::send_turn(race, net)?;
        crate::net::progress::send(race, net);
        apply_received::apply_received(race, net)
    } else if net.race_slot.is_bot(race.shared.work().current_car) && net.vars.own.state().is_host()
    {
        // The host's computer driver: its state goes to the others.
        crate::net::send_bot_turn::send_bot_turn(race, net)
    } else {
        follow_remote::follow_remote(race, net)?;
        Ok(())
    }
}
