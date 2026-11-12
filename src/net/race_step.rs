//! `net::race_step` (0x464133): a car's step in a network race (nothing without one,
//! `session` 0): for the own player's car (`g_currentCar` = `player`) its
//! messages sent (`net::send_turn`) and the others' applied (`net::apply_received`); for
//! another's, its glide (`net::follow_remote`).
//!
//!
//! See `net/mod.rs`. One caller, `drive::replay_step` (each car's step, after its
//! physics).

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::{apply_received, follow_remote, send_turn};
use crate::race::owner::Race;
pub fn race_step(race: &Race, net: &Net) -> Result<(), Refused> {
    let vars = &net.vars;
    if vars.session.get() == 0 {
        return Ok(());
    }
    if race.shared.work.get().current_car.get() == vars.player.get() {
        send_turn::send_turn(race, net)?;
        apply_received::apply_received(race, vars)
    } else {
        follow_remote::follow_remote(race, vars)?;
        Ok(())
    }
}
