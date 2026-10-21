//! The race cars' lights and parts' sounds (converted 2026-10-21 from
//! `race::car_light_modes` 0x455B70, `race::car_lights` 0x455CFF and
//! `race::wake_part_sounds` 0x45D1EA).

use crate::Invalid;

/// `race::car_light_modes`: whether a race car's lights are drawn: it has lights
/// (+0x25C), it is drawn (any car in modes 0 and 2, else the shown car only), and, for
/// the shown car, it is not standing (`standing`: +0x69C and +0x684 both 0) nor hidden
/// (`hidden`: `dword_63993C`); each asked only when it decides.
pub fn lights_drawn(
    has_lights: bool,
    mode: u32,
    shown: bool,
    standing: impl FnOnce() -> bool,
    hidden: impl FnOnce() -> bool,
) -> bool {
    has_lights
        && (mode == 2 || mode == 0 || shown)
        && !(shown && standing())
        && !(shown && hidden())
}

/// A nibble-wise step: `prev`'s bits in `keep` and `bits`.
fn set(prev: u32, keep: u32, bits: u32) -> u32 {
    (prev & keep) | bits
}

/// `race::car_light_modes`: a drawn car's light word (+0x6FC) from `prev`, its nibbles set
/// step by step: 0x21; for the shown car with beams (`dword_606274`), 0x12, then 0x14
/// unless the kind (`dword_606270`, asked only then) is 1, then 0x11 unless it is 2 or 3;
/// then 0x100 in the third nibble, 0x200 with `high` (`dword_6238B4`; 1: the shown car
/// only).
pub fn light_word(
    prev: u32,
    shown: bool,
    beams: impl FnOnce() -> bool,
    kind: impl FnOnce() -> u32,
    high: u32,
) -> u32 {
    let mut w = set(prev, 0xFFFF_FF0F, 0x20);
    w = set(w, 0xFFFF_FFF0, 1);
    if shown && beams() {
        w = set(w, 0xFFFF_FF0F, 0x10);
        w = set(w, 0xFFFF_FFF0, 2);
        let kind = kind();
        if kind != 1 {
            w = set(w, 0xFFFF_FFF0, 4);
            if kind != 2 && kind != 3 {
                w = set(w, 0xFFFF_FFF0, 1);
            }
        }
    }
    w = set(w, 0xFFFF_F0FF, 0x100);
    if high != 0 && (high != 1 || shown) {
        w = set(w, 0xFFFF_F0FF, 0x200);
    }
    w
}

/// The light word's bit for lights on (`light_word` sets it in every drawn car's).
pub const LIGHTS_ON: u32 = 0x20;

/// `race::car_lights`: whether a car's lights are drawn from its light word (+0x6FC).
pub fn lights_on(word: u32) -> bool {
    word & LIGHTS_ON != 0
}

/// The depth entries one car's lights may add (its own and its flares).
pub const ENTRIES_A_CAR: u32 = 25;

/// What `race::car_lights` refuses before anything: no race cars (its loop tests at its
/// end) or more than eight, and a depth list (`count` of `room`) without room for every
/// car's entries.
pub fn check_car_lights(cars: u32, count: u32, room: u32) -> Result<(), Invalid> {
    if cars == 0 || cars > 8 {
        return Err(Invalid("no race cars or more than eight"));
    }
    if count > room - ENTRIES_A_CAR * cars {
        return Err(Invalid("the depth list has no room for the cars' lights"));
    }
    Ok(())
}

/// The count from which a car part's sound wakes (`race::wake_part_sounds`).
pub const WAKE_COUNT: u32 = 10;

/// What `race::wake_part_sounds` does with one part slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartWake {
    /// Already awake (+0x48 set) or not yet counted to 10.
    Left,
    /// +0x48 set to 1.
    Woken,
    /// +0x48 set, and the count exactly 10: the car's sound object may wake too.
    WokenNow,
}

/// `race::wake_part_sounds`: a part slot by its flag (+0x48) and its count (+0x700).
pub fn part_wake(flag: u32, count: u32) -> PartWake {
    if flag != 0 || count < WAKE_COUNT {
        PartWake::Left
    } else if count == WAKE_COUNT {
        PartWake::WokenNow
    } else {
        PartWake::Woken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawn() {
        assert!(lights_drawn(
            true,
            0,
            false,
            || unreachable!(),
            || unreachable!()
        ));
        assert!(!lights_drawn(
            true,
            1,
            false,
            || unreachable!(),
            || unreachable!()
        ));
        assert!(!lights_drawn(
            false,
            0,
            true,
            || unreachable!(),
            || unreachable!()
        ));
        assert!(!lights_drawn(true, 1, true, || true, || unreachable!()));
        assert!(!lights_drawn(true, 1, true, || false, || true));
        assert!(lights_drawn(true, 1, true, || false, || false));
    }

    #[test]
    fn words() {
        assert_eq!(
            light_word(0, false, || unreachable!(), || unreachable!(), 0),
            0x121
        );
        assert_eq!(
            light_word(0xF000_0FFF, false, || unreachable!(), || unreachable!(), 1),
            0xF000_0121
        );
        assert_eq!(light_word(0, true, || true, || 1, 1), 0x212);
        assert_eq!(light_word(0, true, || true, || 2, 0), 0x114);
        assert_eq!(light_word(0, true, || true, || 7, 2), 0x211);
        assert_eq!(light_word(0, true, || false, || unreachable!(), 0), 0x121);
    }

    #[test]
    fn car_lights() {
        assert!(lights_on(0x121) && !lights_on(0x111));
        assert!(check_car_lights(0, 0, 0x1000).is_err());
        assert!(check_car_lights(9, 0, 0x1000).is_err());
        assert!(check_car_lights(8, 0x1000 - 200, 0x1000).is_ok());
        assert!(check_car_lights(8, 0x1000 - 199, 0x1000).is_err());
    }

    #[test]
    fn parts() {
        assert_eq!(part_wake(1, 20), PartWake::Left);
        assert_eq!(part_wake(0, 9), PartWake::Left);
        assert_eq!(part_wake(0, 10), PartWake::WokenNow);
        assert_eq!(part_wake(0, 11), PartWake::Woken);
    }
}
