//! A player's record, the 16 bytes of DirectPlay player data each machine keeps for its
//! player (from `net::fill_own_player` 0x4181A3, `net::set_up_race`
//! 0x463A94, `net::lets_rally` 0x4170E0, `net::close_if_host` 0x41837B and
//! `net::lets_race` 0x4640AA): +0 the state (`crate::net::players`' bits), +1 the car, +2
//! its colours, +3 the nationality, +4 the stage, +5 its direction, +6 the race number, +7
//! a random byte, +8 the race's time, +0xC the total; and what a race is set up from it.

use crate::Invalid;
use crate::net::players::HOST;

pub const STATE: usize = 0;
pub const CAR: usize = 1;
pub const COLOURS: usize = 2;
pub const NATIONALITY: usize = 3;
pub const STAGE: usize = 4;
pub const DIRECTION: usize = 5;
pub const RACE_NUMBER: usize = 6;
pub const RANDOM: usize = 7;

/// The stage a record names when none was chosen (the chosen stage 0), with direction 0.
pub const DEFAULT_STAGE: u8 = 0x29;

/// This machine's player as its record starts (`fill_own_player`): what the record's
/// first seven bytes become, the state 0; the random byte goes in after.
pub fn own_record(
    car: u8,
    colours: u8,
    nationality: u8,
    stage: u8,
    direction: u8,
    race_number: u8,
) -> [u8; RANDOM] {
    let (stage, direction) = if stage == 0 {
        (DEFAULT_STAGE, 0)
    } else {
        (stage, direction)
    };
    let mut out = [0u8; RANDOM];
    out[CAR] = car;
    out[COLOURS] = colours;
    out[NATIONALITY] = nationality;
    out[STAGE] = stage;
    out[DIRECTION] = direction;
    out[RACE_NUMBER] = race_number;
    out
}

/// Whether a record's state is the host's.
pub fn is_host(state: u8) -> bool {
    state & HOST != 0
}

/// The most players a race's tables hold.
pub const RACE_PLAYERS: u32 = 8;

/// A race's players: refused, none or more than the tables' eight (the original's loop,
/// ended by `jnz`, runs past them).
pub fn race_players(players: u32) -> Result<u32, Invalid> {
    if players == 0 || players > RACE_PLAYERS {
        return Err(Invalid("no players, or more than eight"));
    }
    Ok(players)
}

/// A player's place in the race (`set_up_race`), from its record (`byte`, asked of the
/// offsets the original reads, the direction only for the host).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seat {
    pub car: u32,
    pub colours: u32,
    pub race_number: u32,
    /// The host's stage (low seven bits) and direction (bit 0).
    pub stage: Option<(u32, u32)>,
}

pub fn seat(byte: impl Fn(usize) -> u8) -> Seat {
    let car = byte(CAR) as u32;
    let colours = byte(COLOURS) as u32;
    let race_number = byte(RACE_NUMBER) as u32;
    let stage = byte(STAGE) as u32;
    let stage = is_host(byte(STATE)).then(|| (stage & 0x7F, byte(DIRECTION) as u32 & 1));
    Seat {
        car,
        colours,
        race_number,
        stage,
    }
}

/// The first player that is not the host among `count` (`state_of` each in turn; the first
/// looked at even with none), or `None`.
pub fn first_not_host(count: u32, state_of: impl Fn(u32) -> u8) -> Option<u32> {
    let mut n = 0u32;
    loop {
        if !is_host(state_of(n)) {
            return Some(n);
        }
        n = n.wrapping_add(1);
        if n >= count {
            return None;
        }
    }
}

/// What a car's block's +0xB0 says of its race (`lets_race`'s ebx): -1 not finished, else
/// 0.
pub fn not_finished(flag: u32) -> u32 {
    if flag == u32::MAX { u32::MAX } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records() {
        assert_eq!(own_record(3, 4, 5, 7, 1, 9), [0, 3, 4, 5, 7, 1, 9]);
        assert_eq!(own_record(3, 4, 5, 0, 1, 9), [0, 3, 4, 5, 0x29, 0, 9]);
        assert!(is_host(0x11) && !is_host(0x01));
        assert!(race_players(0).is_err());
        assert!(race_players(9).is_err());
        assert_eq!(race_players(8), Ok(8));
        let host = [0x10u8, 5, 6, 0, 0x85, 3, 7, 0];
        assert_eq!(
            seat(|n| host[n]),
            Seat {
                car: 5,
                colours: 6,
                race_number: 7,
                stage: Some((5, 1))
            }
        );
        let other = [0u8, 5, 6, 0, 0x85, 3, 7, 0];
        assert_eq!(seat(|n| other[n]).stage, None);
        let states = [0x10u8, 0x10, 1];
        assert_eq!(first_not_host(3, |n| states[n as usize]), Some(2));
        assert_eq!(first_not_host(2, |n| states[n as usize]), None);
        assert_eq!(first_not_host(0, |n| states[n as usize]), None);
        assert_eq!(first_not_host(0, |_| 0), Some(0));
        assert_eq!(not_finished(u32::MAX), u32::MAX);
        assert_eq!(not_finished(3), 0);
    }
}
