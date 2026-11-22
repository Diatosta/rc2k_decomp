//! The own car's messages by its step count: every fourth step its inputs, at step 2 of 32 its
//! position, at step 1 of 256 the extra dwords.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::{send_extra, send_inputs, send_position};
use crate::race::owner::Race;
use rc2k_core::net::race::SENT;
/// One of the three messages sent.
type Send = fn(&Race, &Net) -> Result<(), Refused>;

pub(super) fn send_turn(race: &Race, net: &Net) -> Result<(), Refused> {
    let sends: [Send; 3] = [
        send_inputs::send_inputs,
        send_position::send_position,
        send_extra::send_extra,
    ];
    // The logic is in `rc2k_core::net::race::Kind::due`.
    for (kind, send) in SENT.into_iter().zip(sends) {
        if kind.due(race.shared.car().step_clock).0 {
            send(race, net)?;
        }
    }
    Ok(())
}
