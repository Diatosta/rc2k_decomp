//! The two lists of cars that can be chosen (converted 2026-10-21 from
//! `saves::rebuild_car_lists` 0x47088C and `saves::list_car_numbers` 0x470825): kept in
//! `var\ral.dat` as numbers, used as pointers to the cars' records.

use crate::Invalid;

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
    }
}
