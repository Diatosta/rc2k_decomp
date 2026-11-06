//! `net::race_step` (0x464133): a car's step in a network race (nothing without one,
//! `dword_52A910` 0): for the own player's car (`g_currentCar` = `dword_52A920`) its
//! messages sent (`net::send_turn`) and the others' applied (`net::apply_received`); for
//! another's, its glide (`net::follow_remote`).
//!
//! In:  nothing
//! Out: nothing (the original leaves eax `g_currentCar`, or as the calls leave it; its
//!      caller, `drive::replay_step`'s car step, runs it on a copy of its registers and
//!      reads none of them after it)
//!
//! See `net/mod.rs`. One caller, `drive::replay_step` (each car's step, after its
//! physics).

use crate::errors::Refused;
use crate::net::owner::net;
use crate::net::{apply_received, follow_remote, send_turn};
use crate::race::owner::race;

/// The Rust version.
pub fn race_step() -> Result<(), Refused> {
    let vars = &net().vars;
    if vars.session.get() == 0 {
        return Ok(());
    }
    if race().shared.work.current_car.get() == vars.player.get() {
        send_turn::send_turn()?;
        apply_received::apply_received()
    } else {
        follow_remote::follow_remote()?;
        Ok(())
    }
}
