//! Puts one message received from a player into that player's car block, logging the step and
//! the sender first. Players 8 and up are ignored.
//! - 'JGSM': the four input dwords.
//! - 'XGSM': nine extra dwords.
//! - 'PGSM': the position the car should be at, then the remaining dwords. After the car's first
//!   1000 steps, if the received position is 10 or more away from the car's, the car jumps there;
//!   otherwise the target is set to a 32nd of the way and 32 glide steps are armed for
//!   `net::follow_remote`.
//! - Anything else: nothing.

use crate::errors::BadInput;
use crate::logfile::log_int::log_int;
use crate::net::owner::Net;
use crate::net::send_inputs::fields;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use rc2k_core::net::message::Payload;
use rc2k_core::net::race::{self, Arrival, Kind};

/// The log text before the sender's number.
const TEXT: &std::ffi::CStr = c"rcvd fm plyr";

/// A car's place (`position`).
pub fn place(car: &Car) -> &[f32; 3] {
    &car.position
}

/// A car's place, to write.
pub fn place_mut(car: &mut Car) -> &mut [f32; 3] {
    &mut car.position
}

/// Where a place received goes, to be glided to (`glide_target`).
pub fn wanted(car: &Car) -> &[f32; 3] {
    &car.glide_target
}

/// Where a place received goes, to write.
pub fn wanted_mut(car: &mut Car) -> &mut [f32; 3] {
    &mut car.glide_target
}

/// A message no car could send, told once in the log.
fn note_unbelievable(player: u32) {
    static TOLD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !TOLD.swap(true, std::sync::atomic::Ordering::Relaxed) {
        crate::program_log::log(&format!(
            "net: player {player}'s message has values no car could hold: not applied"
        ));
    }
}

pub fn apply_message(race: &Race, net: &Net, player: u32, message: &[u8]) -> Result<(), BadInput> {
    log_int(net.race_slot.player() as i32, false, 1);
    log_int(race.shared.view.get().local_car.get() as i32, false, 1);
    log_int(
        *message.first().ok_or(BadInput("a received message"))? as i32,
        false,
        1,
    );
    crate::logfile::log_write::log_nul(&TEXT, false, 1);
    log_int(player as i32, true, 1);
    if player >= 8 {
        return Ok(());
    }
    // The block is looked up after the message is read and believed: a null one refuses at the
    // first write.
    let (kind, values) = match Payload::parse(message) {
        Payload::Fields { kind, values, .. } => (kind, values),
        Payload::Unknown(_) => return Ok(()),
        Payload::Truncated(_) => return Err(BadInput("a received message")),
    };
    let table = fields(kind);
    if !race::believable(table.iter().map(|&(_, limit)| limit), &values) {
        note_unbelievable(player);
        return Ok(());
    }
    let mut car = race.shared.race_car_mut(player as usize)?;
    for (n, (&(field, _), &value)) in table.iter().zip(&values).enumerate() {
        // The place received goes where the car is to be.
        if kind == Kind::Position && n < 3 {
            wanted_mut(&mut car)[n] = f32::from_bits(value);
        } else {
            field(&mut car).set_bits(value);
        }
    }
    if kind != Kind::Position {
        return Ok(());
    }
    let wanted_place = bits3(wanted(&car));
    let now = bits3(place(&car));
    let steps = car.step_clock;
    match race::arrival(wanted_place, now, steps, race::SNAP_FROM) {
        Arrival::Jump => {
            *place_mut(&mut car) = wanted_place.map(f32::from_bits);
            // The glide target now holds a place, not a step: a glide under way ends.
            car.glide_steps_left = 0;
        }
        Arrival::Glide(step) => {
            *wanted_mut(&mut car) = step.map(f32::from_bits);
            car.glide_steps_left = race::GLIDE;
        }
    }
    Ok(())
}

/// The bits of a place's three floats.
fn bits3(place: &[f32; 3]) -> [u32; 3] {
    place.map(f32::to_bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The places sit where the core's constants put them in the car's block.
    #[test]
    fn the_places_sit_where_the_offsets_were() {
        use crate::physics::car_offsets_1999::offset_changed_by;
        assert_eq!(
            offset_changed_by(|c| place_mut(c)[0] = f32::from_bits(1)),
            0x134
        );
        assert_eq!(
            offset_changed_by(|c| wanted_mut(c)[0] = f32::from_bits(1)),
            0xFC0
        );
        assert_eq!(offset_changed_by(|c| c.glide_steps_left = 1), 0xFCC);
        assert_eq!(offset_changed_by(|c| c.step_clock = 1), 8);
    }
}
