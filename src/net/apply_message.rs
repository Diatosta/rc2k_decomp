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
use crate::net::owner::net_race;
use crate::net::send_inputs::fields;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::session::PlainDword;
use rc2k_core::net::message::Payload;
use rc2k_core::net::race::{self, Arrival, Kind};

/// The log text before the sender's number.
const TEXT: &std::ffi::CStr = c"rcvd fm plyr"; // "rcvd fm plyr"

/// A car's place (+0x134 .. +0x13C).
pub fn place(car: &Car) -> &[PlainDword; 3] {
    &car.position
}

/// Where a place received goes, to be glided to (+0xFC0 .. +0xFC8; then the glide's 32nd).
pub fn wanted(car: &Car) -> &[PlainDword] {
    &car.glide_target
}

/// The glide's steps left (+0xFCC).
pub fn glide_steps(car: &Car) -> &PlainDword {
    &car.glide_steps_left
}

/// The three dwords' values.
pub fn three(cells: &[PlainDword]) -> [u32; 3] {
    [cells[0].get(), cells[1].get(), cells[2].get()]
}

/// A message no car could send, told once in the log.
fn note_unbelievable(player: u32) {
    static TOLD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !TOLD.swap(true, std::sync::atomic::Ordering::Relaxed) {
        crate::framework::log(&format!(
            "net: player {player}'s message has values no car could hold: not applied"
        ));
    }
}

pub fn apply_message(race: &Race, player: u32, message: &[u8]) -> Result<(), BadInput> {
    log_int(net_race().player() as i32, false, 1);
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
    let car = race.shared.race_car(player as usize)?;
    for (n, (&(field, _), &value)) in table.iter().zip(&values).enumerate() {
        // The place received goes where the car is to be.
        if kind == Kind::Position && n < 3 {
            wanted(car)[n].set(value);
        } else {
            field(car).set_bits(value);
        }
    }
    if kind != Kind::Position {
        return Ok(());
    }
    let (wanted_place, now) = (three(wanted(car)), three(place(car)));
    let steps = car.step_clock.get();
    match race::arrival(wanted_place, now, steps, race::SNAP_FROM) {
        Arrival::Jump => {
            for (cell, v) in place(car).iter().zip(wanted_place) {
                cell.set(v);
            }
            // +0xFC0 now holds a place, not a step: a glide under way ends.
            glide_steps(car).set(0);
        }
        Arrival::Glide(step) => {
            for (cell, v) in wanted(car).iter().zip(step) {
                cell.set(v);
            }
            glide_steps(car).set(race::GLIDE);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The places sit where the core's constants put them in the car's block.
    #[test]
    fn the_places_sit_where_the_offsets_were() {
        let car = crate::test_support::own::race().shared.car();
        let at = |cell: &PlainDword| {
            crate::physics::car_offsets_1999::offset_1999(car, core::ptr::from_ref(cell).addr())
                as u32
        };
        assert_eq!(at(&place(car)[0]), 0x134);
        assert_eq!(at(&wanted(car)[0]), 0xFC0);
        assert_eq!(at(glide_steps(car)), 0xFCC);
        assert_eq!(at(&car.step_clock), 8);
    }
}
