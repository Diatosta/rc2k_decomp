//! `net::apply_message` (0x4644EC): one message received from a player put into that
//! player's car block (`g_carBlocks[player]`), after logging the own player, the step, the
//! message's first byte, "rcvd fm plyr" and the player's number. Players 8 and up: nothing more.
//! - 'JGSM': the four input dwords into +0xFD0 .. +0xFDC;
//! - 'XGSM': nine dwords into +0x64 .. +0x84;
//! - 'PGSM': the place into +0xFC0 .. +0xFC8 (where the car is to be), the other 37
//!   dwords where `net::send_position` took them from. Then, after the car's first 1000
//!   steps (+8) and when the place received is 10 or more away from the
//!   car's (+0x134 .. +0x13C, whole numbers; a distance by square root) the
//!   car is put there; else +0xFC0 .. +0xFC8 become a 32nd of the way and
//!   +0xFCC 32, for `net::follow_remote` to glide it;
//! - others: nothing.
//!
//! Takes `player`, `message`.
//!
//! See `net/mod.rs`. One caller, `net::apply_received`.

use crate::errors::BadInput;
use crate::logfile::log_int::log_int;
use crate::net::send_inputs::fields;
use crate::net::store::NetVars;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::session::Dword;
use rc2k_core::net::race::{self, Arrival, Kind};

/// g_carBlocks: the eight cars' blocks.
const TEXT: &std::ffi::CStr = c"rcvd fm plyr"; // "rcvd fm plyr"

/// A car's place (+0x134 .. +0x13C).
pub fn place(car: &Car) -> &[Dword; 3] {
    &car.position
}

/// Where a place received goes, to be glided to (+0xFC0 .. +0xFC8; then the glide's 32nd).
pub fn wanted(car: &Car) -> &[Dword] {
    &car.glide_target
}

/// The glide's steps left (+0xFCC).
pub fn glide_steps(car: &Car) -> &Dword {
    &car.glide_steps_left
}

/// The three dwords' values.
pub fn three(cells: &[Dword]) -> [u32; 3] {
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

/// The dword at byte `at` of the message.
fn word_at(message: &[u8], at: usize) -> Result<u32, BadInput> {
    let word = message
        .get(at..at + 4)
        .ok_or(BadInput("a received message"))?;
    Ok(u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
}
pub fn apply_message(
    race: &Race,
    vars: &NetVars,
    player: u32,
    message: &[u8],
) -> Result<(), BadInput> {
    log_int(vars.player.get() as i32, false, 1);
    log_int(race.shared.view.local_car.get() as i32, false, 1);
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
    // The block is looked up after the message is read and believed: a null one refuses
    // at the first write, as the original's.
    let Some(kind) = Kind::of(word_at(message, 0)?) else {
        return Ok(());
    };
    let table = fields(kind);
    let mut values = Vec::with_capacity(table.len());
    for n in 0..table.len() as u32 {
        values.push(word_at(message, 4 + 4 * n as usize)?);
    }
    if !race::believable(table.iter().map(|&(_, limit)| limit), &values) {
        note_unbelievable(player);
        return Ok(());
    }
    let car = race.shared.work.car(player as usize)?;
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
    use crate::race::owner::race;

    /// The places sit where the core's constants put them in the car's block.
    #[test]
    fn the_places_sit_where_the_offsets_were() {
        let car = &race().shared.car;
        let base = core::ptr::from_ref(car).addr();
        let at = |cell: &Dword| (core::ptr::from_ref(cell).addr() - base) as u32;
        assert_eq!(at(&place(car)[0]), 0x134);
        assert_eq!(at(&wanted(car)[0]), 0xFC0);
        assert_eq!(at(glide_steps(car)), 0xFCC);
        assert_eq!(at(&car.step_clock), 8);
    }
}
