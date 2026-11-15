//! A car's step in a network race (nothing without a session): the own player's car sends its
//! messages and applies the others'; another car follows its glide. Called from each car's step
//! after its physics.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::{apply_received, follow_remote, send_turn};
use crate::race::owner::Race;
pub fn race_step(race: &Race, net: &Net) -> Result<(), Refused> {
    if !net.race_slot.is_on() {
        return Ok(());
    }
    if race.shared.work.get().current_car.get() == net.race_slot.player() {
        send_turn::send_turn(race, net)?;
        apply_received::apply_received(race, net)
    } else {
        follow_remote::follow_remote(race, net)?;
        Ok(())
    }
}
