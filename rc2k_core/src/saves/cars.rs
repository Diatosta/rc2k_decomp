//! The two lists of cars that can be chosen (converted 2026-10-21 from
//! `saves::rebuild_car_lists` 0x47088C and `saves::list_car_numbers` 0x470825): kept in
//! `var\ral.dat` as numbers, used as pointers to the cars' records; and what a loaded
//! game's cars keep of the players' own settings (converted 2026-10-21 from
//! `saves::keep_player_settings` 0x4719F5).

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// A saved game's car (a player's object as it is saved).
pub const SAVED_CAR_LEN: usize = 0x28C;

/// The players' own settings that a loaded game keeps (the controls and choices made
/// since it was saved), as (offset, bytes): the keys' bytes +0x1C .. +0x27, the dwords
/// +0x2C .. +0x78 and +0x178.
pub const KEPT_SETTINGS: [(usize, usize); 3] = [(0x1C, 0xC), (0x2C, 0x50), (0x178, 4)];

/// `saves::keep_player_settings` for one player: its kept settings (`player`'s bytes)
/// written over the saved car's (`car`, by offset). Refused: a read past the player.
pub fn keep_settings(
    player: &(impl Bytes + ?Sized),
    car: &mut impl FnMut(usize, u8),
) -> Result<(), Invalid> {
    for (start, len) in KEPT_SETTINGS {
        for at in start..start + len {
            car(
                at,
                player
                    .byte(at)
                    .ok_or(Invalid("a player's settings cut short"))?,
            );
        }
    }
    Ok(())
}

/// The car numbers' limit (the original reads wherever a bigger one leads).
pub const MOST_CARS: u32 = 0x400;

/// A list of pointers from a list of numbers up to its -1 (`car` the record of a
/// number), the rest -1. Refused: no -1 within the list, a number of `MOST_CARS` or more.
pub fn pointers(numbers: &[u32], car: impl Fn(u32) -> u32) -> Result<Vec<u32>, Invalid> {
    let ids: Vec<u32> = numbers
        .iter()
        .copied()
        .take_while(|&id| id != u32::MAX)
        .collect();
    if ids.len() == numbers.len() {
        return Err(Invalid("a list of car numbers without its end"));
    }
    if ids.iter().any(|&id| id >= MOST_CARS) {
        return Err(Invalid("a car number far past the table"));
    }
    let mut out: Vec<u32> = ids.into_iter().map(car).collect();
    out.resize(numbers.len(), u32::MAX);
    Ok(out)
}

/// Whether a list of numbers in the options file was never filled: all 0 and no -1, as a file
/// saved before the front end first ended has it (never a real list, which has its -1).
pub fn never_listed(numbers: &[u32]) -> bool {
    numbers.iter().all(|&id| id == 0)
}

/// A list of numbers from a list of pointers up to its -1 (`number` a record's first
/// dword), the rest -1. Refused: no -1 within the list, a null pointer.
pub fn numbers(pointers: &[u32], number: impl Fn(u32) -> u32) -> Result<Vec<u32>, Invalid> {
    let cars: Vec<u32> = pointers
        .iter()
        .copied()
        .take_while(|&c| c != u32::MAX)
        .collect();
    if cars.len() == pointers.len() {
        return Err(Invalid("a list of cars without its end"));
    }
    if cars.contains(&0) {
        return Err(Invalid("a null car in a list"));
    }
    let mut out: Vec<u32> = cars.into_iter().map(number).collect();
    out.resize(pointers.len(), u32::MAX);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists() {
        assert_eq!(
            pointers(&[2, 1, u32::MAX, 7], |n| 0x100 + n),
            Ok(vec![0x102, 0x101, u32::MAX, u32::MAX])
        );
        assert!(pointers(&[1, 2], |n| n).is_err() && pointers(&[0x400, u32::MAX], |n| n).is_err());
        assert_eq!(
            numbers(&[0x102, u32::MAX], |p| p - 0x100),
            Ok(vec![2, u32::MAX])
        );
        assert!(numbers(&[0, u32::MAX], |p| p).is_err());
        assert!(never_listed(&[0, 0]));
        assert!(!never_listed(&[0, u32::MAX]) && !never_listed(&[1, 0]));
    }

    #[test]
    fn kept_settings() {
        let player: Vec<u8> = (0..SAVED_CAR_LEN).map(|n| n as u8).collect();
        let mut car = vec![0xAAu8; SAVED_CAR_LEN];
        keep_settings(&player, &mut |at, b| car[at] = b).unwrap();
        assert_eq!(
            (car[0x1B], car[0x1C], car[0x27], car[0x28]),
            (0xAA, 0x1C, 0x27, 0xAA)
        );
        assert_eq!(
            (car[0x2B], car[0x2C], car[0x7B], car[0x7C]),
            (0xAA, 0x2C, 0x7B, 0xAA)
        );
        assert_eq!(
            (car[0x177], car[0x178], car[0x17B], car[0x17C]),
            (0xAA, 0x78, 0x7B, 0xAA)
        );
    }
}
