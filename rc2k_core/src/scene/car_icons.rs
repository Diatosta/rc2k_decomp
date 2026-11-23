//! The HUD's two car icons, each chosen by its model's icon number and uploaded to one of two
//! rows of the HUD's texture.

use crate::Invalid;

/// The icon tables' room (0x90 bytes each).
pub const NUMBERS: u32 = 0x24;
/// Icon numbers below this use the first pair of pictures, the rest the second.
pub const FIRST_PAIR_BELOW: u32 = 0x12;
/// The two icons' rows on the HUD's texture.
pub const ROWS: [u32; 2] = [0x68, 0x94];
/// The icons' column on the HUD's texture.
pub const COLUMN: u32 = 0xAD;
/// The most cars.
pub const CARS: u32 = 8;
/// A full-size icon's width and height (the sheets' 82 by 42).
pub const FULL_SIZE: [u32; 2] = [0x52, 0x2A];
/// With three or four parts every part's car icon is painted at half size (a quarter of the
/// area): an icon's width and height there. The first four cars' share the first row's place, the
/// fifth to the eighth car's (a replay's second group) the second row's, which a split screen
/// leaves free (it shows no second full-size icon).
pub const HALF_SIZE: [u32; 2] = [0x29, 0x15];
/// How many half-size icons one row's place holds.
pub const HALF_PER_ROW: usize = 4;

/// Where car `n`'s half-size icon lies on the HUD's texture: left and top; none past the eighth.
pub fn half_place(n: usize) -> Option<[u32; 2]> {
    let row = ROWS.get(n / HALF_PER_ROW)?;
    let k = (n % HALF_PER_ROW) as u32;
    Some([
        COLUMN + (k % 2) * HALF_SIZE[0],
        row + (k / 2) * HALF_SIZE[1],
    ])
}

/// Refuses an icon number past the tables, before anything is uploaded.
pub fn check_numbers(numbers: [u32; 2]) -> Result<(), Invalid> {
    if numbers.iter().any(|&n| n >= NUMBERS) {
        return Err(Invalid("an icon's number past its tables"));
    }
    Ok(())
}

/// The first of an icon's pair of pictures (0 or 2 of the four names).
pub fn pictures(number: u32) -> usize {
    if number < FIRST_PAIR_BELOW { 0 } else { 2 }
}

/// The one car shown: its block's number; refuses one past the blocks.
pub fn one_car(car: u32) -> Result<usize, Invalid> {
    if car >= CARS {
        return Err(Invalid("the one car's number is past the blocks"));
    }
    Ok(car as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_icons_fill_the_rows() {
        assert_eq!(half_place(0), Some([0xAD, 0x68]));
        assert_eq!(half_place(3), Some([0xAD + 0x29, 0x68 + 0x15]));
        assert_eq!(half_place(4), Some([0xAD, 0x94]));
        assert_eq!(half_place(7), Some([0xAD + 0x29, 0x94 + 0x15]));
        assert_eq!(half_place(8), None);
        // Each row's four within the place of a full-size icon, the rows apart.
        for (n, row) in [(3, ROWS[0]), (7, ROWS[1])] {
            let [x, y] = half_place(n).unwrap();
            assert!(x + HALF_SIZE[0] <= COLUMN + FULL_SIZE[0]);
            assert!(y + HALF_SIZE[1] <= row + FULL_SIZE[1]);
        }
        assert!(ROWS[0] + FULL_SIZE[1] <= ROWS[1]);
    }

    #[test]
    fn icons() {
        assert!(check_numbers([0, 0x24]).is_err());
        assert!(check_numbers([0x23, 0]).is_ok());
        assert_eq!((pictures(0x11), pictures(0x12)), (0, 2));
        assert!(one_car(8).is_err());
        assert_eq!(one_car(7), Ok(7));
    }
}
