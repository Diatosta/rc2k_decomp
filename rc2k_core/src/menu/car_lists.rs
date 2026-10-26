//! The lists of cars that can be chosen, and what the cheats add to them (converted
//! 2026-10-21 from `menu::remove_secret_cars` 0x471BC7, `menu::add_clio_car` 0x471C33 and
//! `menu::apply_cheats` 0x480AC1, and the choice of the cheat's list from
//! `menu::cheat_car_list` 0x480D35; the arcade's `menu::add_shown_car` 0x486BD2). A list holds the addresses of car descriptions and
//! ends with -1; each has a room of so many dwords before the next variable.

use crate::Invalid;

/// A list's end mark.
pub const END: u32 = u32::MAX;

/// A list's entries before its end (`word` its dwords, asked in order up to the end).
/// Refused: no end within `room`.
pub fn entries(room: usize, word: impl Fn(usize) -> u32) -> Result<Vec<u32>, Invalid> {
    let cars: Vec<u32> = (0..room).map(word).take_while(|&e| e != END).collect();
    if cars.len() == room {
        return Err(Invalid("a list of cars without its end"));
    }
    Ok(cars)
}

/// `menu::add_clio_car`, for one list: where the car goes (over the end mark, with no new
/// end after it: the original's slip, kept), none when the list has it.
pub fn place_if_missing(cars: &[u32], car: u32) -> Option<usize> {
    (!cars.contains(&car)).then_some(cars.len())
}

/// `menu::add_shown_car`: the shown car, entry `shown` of the shown list; it goes over
/// the cars' list's end mark, with no new end after it (as `add_clio_car`). Refused: a
/// shown car past its list.
pub fn shown_car(shown_list: &[u32], shown: u32) -> Result<u32, Invalid> {
    shown_list
        .get(shown as usize)
        .copied()
        .ok_or(Invalid("a shown car past the list"))
}

/// `menu::remove_secret_cars`: each list without the secret cars. Refused: a list left with
/// no car (the original copies one entry back and then counts through all 32 bits), more
/// cars than the scratch list (`scratch`) has room for.
pub fn without_secret(
    lists: &[Vec<u32>],
    secret: &[u32],
    scratch: usize,
) -> Result<Vec<Vec<u32>>, Invalid> {
    lists
        .iter()
        .map(|cars| {
            let kept: Vec<u32> = cars
                .iter()
                .copied()
                .filter(|car| !secret.contains(car))
                .collect();
            if kept.is_empty() {
                return Err(Invalid("a list of cars with none that is not secret"));
            }
            if kept.len() > scratch {
                return Err(Invalid("more cars than the scratch list has room for"));
            }
            Ok(kept)
        })
        .collect()
}

/// The car added to a list of `room` dwords unless it is there; true if it was added.
/// Refused: no room for the car and the end after it.
pub fn add_car(cars: &mut Vec<u32>, room: usize, car: u32) -> Result<bool, Invalid> {
    if cars.contains(&car) {
        return Ok(false);
    }
    if cars.len() + 2 > room {
        return Err(Invalid("a list of cars without room for one more"));
    }
    cars.push(car);
    Ok(true)
}

/// `menu::cheat_car_list`: whether the cheat's car list (`off_75A412`) is chosen: the
/// cheat "Unlock Cars" (bit 1) on.
pub fn cheat_cars(cheats: u32) -> bool {
    cheats & 1 << 1 != 0
}

/// What a cheat (a bit of `g_cheats`) does besides being logged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cheat {
    /// The bit stays set (other code looks at it).
    Stays,
    /// The bit is cleared.
    Cleared,
    /// The bit is cleared and its car added to the first list; `true`: then, when it was
    /// added there, to the second too.
    Car(bool),
    /// The bit stays set and `dword_75ACCC` becomes 1.
    Stores,
}

/// By bit (see `menu::apply_cheats`' hook for their names).
pub const CHEATS: [Cheat; 19] = [
    Cheat::Stays,      // 1 Unlock Arcade
    Cheat::Stays,      // 2 Unlock Cars
    Cheat::Car(false), // 3 Spud
    Cheat::Car(false), // 4 Sheep
    Cheat::Car(false), // 5 Cow
    Cheat::Car(false), // 6 MF Sports
    Cheat::Car(true),  // 7 Max Power
    Cheat::Car(false), // 8 Local Shop
    Cheat::Car(false), // 9 Bolt's Pug
    Cheat::Car(false), // 10 Skip
    Cheat::Cleared,    // 11
    Cheat::Car(false), // 12 MF Hatchback
    Cheat::Car(false), // 13 Sheep Wagon
    Cheat::Stays,      // 14 No Damage
    Cheat::Stays,      // 15 Damage Timer
    Cheat::Stays,      // 16
    Cheat::Stays,      // 17 Arcade Timer
    Cheat::Stores,     // 18 Enable A8 Game
    Cheat::Stores,     // 19 Reverse Sim Tracks
];

/// A list and its room.
pub struct List {
    pub cars: Vec<u32>,
    pub room: usize,
    /// A car was added.
    pub changed: bool,
}

/// What `apply_cheats` leaves: the cheats' bits, whether `dword_75ACCC` becomes 1, and
/// the bits whose lines are logged, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub cheats: u32,
    pub stores: bool,
    pub logged: Vec<usize>,
}

/// `menu::apply_cheats` on the two lists (`car` gives a cheat's car by its bit). Refused
/// (nothing changed for the caller to write): a list without room for a car more.
pub fn apply_cheats(
    cheats: u32,
    first: &mut List,
    second: &mut List,
    car: impl Fn(usize) -> u32,
) -> Result<Applied, Invalid> {
    let mut a = Applied {
        cheats,
        stores: false,
        logged: Vec::new(),
    };
    for (bit, does) in CHEATS.into_iter().enumerate() {
        if a.cheats & (1 << bit) == 0 {
            continue;
        }
        a.logged.push(bit);
        match does {
            Cheat::Stays => {}
            Cheat::Cleared => a.cheats &= !(1 << bit),
            Cheat::Car(both) => {
                a.cheats &= !(1 << bit);
                let car = car(bit);
                if add_car(&mut first.cars, first.room, car)? {
                    first.changed = true;
                    if both && add_car(&mut second.cars, second.room, car)? {
                        second.changed = true;
                    }
                }
            }
            Cheat::Stores => a.stores = true,
        }
    }
    Ok(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists() {
        let words = [5, 6, END, 7];
        assert_eq!(entries(4, |n| words[n]), Ok(vec![5, 6]));
        assert!(entries(2, |n| words[n]).is_err());
        assert!(cheat_cars(2) && !cheat_cars(0xFFFF_FFFD));
        assert_eq!(shown_car(&[7, 8], 1), Ok(8));
        assert!(shown_car(&[7, 8], 2).is_err());
        assert_eq!(place_if_missing(&[5, 6], 6), None);
        assert_eq!(place_if_missing(&[5, 6], 9), Some(2));
        assert_eq!(
            without_secret(&[vec![1, 2, 3], vec![2, 4]], &[2], 5),
            Ok(vec![vec![1, 3], vec![4]])
        );
        assert!(without_secret(&[vec![2]], &[2], 5).is_err());
        assert!(without_secret(&[vec![1, 3]], &[2], 1).is_err());
        let mut cars = vec![1];
        assert_eq!(add_car(&mut cars, 3, 1), Ok(false));
        assert_eq!(add_car(&mut cars, 3, 2), Ok(true));
        assert!(add_car(&mut cars, 3, 4).is_err());
    }

    #[test]
    fn cheats() {
        let mut first = List {
            cars: vec![100],
            room: 10,
            changed: false,
        };
        let mut second = List {
            cars: vec![],
            room: 10,
            changed: false,
        };
        let bits = 1 | 1 << 6 | 1 << 10 | 1 << 17;
        let a = apply_cheats(bits, &mut first, &mut second, |bit| 200 + bit as u32).unwrap();
        assert_eq!(
            a,
            Applied {
                cheats: 1 | 1 << 17,
                stores: true,
                logged: vec![0, 6, 10, 17],
            }
        );
        assert_eq!((first.cars, first.changed), (vec![100, 206], true));
        assert_eq!((second.cars, second.changed), (vec![206], true));
        // Max Power already in the first list: the second is not looked at.
        let mut first = List {
            cars: vec![206],
            room: 10,
            changed: false,
        };
        let mut second = List {
            cars: vec![],
            room: 1,
            changed: false,
        };
        apply_cheats(1 << 6, &mut first, &mut second, |bit| 200 + bit as u32).unwrap();
        assert!(!first.changed && !second.changed);
    }
}
