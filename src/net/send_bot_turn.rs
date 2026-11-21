//! A computer driver's car's messages on the host: its place and motion (every 32 steps) and its
//! extra values (every 256), sent as the host's `Message::BotRace` for the car, so that the
//! joiners' copies of the bot, which they drive themselves, follow the host's as a player's car
//! is followed. Its inputs are not sent: every machine's computer driver works them out.

use crate::errors::Refused;
use crate::memory::session::Cell32;
use crate::net::dplay_send::dplay_send;
use crate::net::owner::Net;
use crate::net::send_inputs::fields;
use crate::race::owner::Race;
use rc2k_core::net::message::{Message, Payload};
use rc2k_core::net::race::Kind;

/// Sends the current (bot) car's messages that are due at its step count.
pub(super) fn send_bot_turn(race: &Race, net: &Net) -> Result<(), Refused> {
    let car = race.shared.car();
    let number = race.shared.work.get().current_car.get() as u8;
    for kind in [Kind::Position, Kind::Extra] {
        if !kind.due(car.step_clock.get()).0 {
            continue;
        }
        let values = fields(kind)
            .iter()
            .map(|(field, _)| field(car).bits())
            .collect();
        let message = Message::BotRace {
            car: number,
            sequence: crate::net::bots::next_sequence(),
            payload: Payload::of(kind, values),
        }
        .encode_for_send()?;
        dplay_send(&net.link, net.link.send_to(), &message);
    }
    Ok(())
}
