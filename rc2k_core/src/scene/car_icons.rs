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
/// With three or four players every part's car icon is painted at half size (a quarter of the
/// area), the four sharing the first row's place: an icon's width and height there.
pub const HALF_SIZE: [u32; 2] = [0x29, 0x15];

/// Where player `n`'s half-size icon lies on the HUD's texture: left and top.
pub fn half_place(n: usize) -> [u32; 2] {
    [
        COLUMN + (n as u32 % 2) * HALF_SIZE[0],
        ROWS[0] + (n as u32 / 2) * HALF_SIZE[1],
    ]
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
    fn half_icons_fill_the_first_row() {
        assert_eq!(half_place(0), [0xAD, 0x68]);
        assert_eq!(half_place(3), [0xAD + 0x29, 0x68 + 0x15]);
        assert!(half_place(3)[0] + HALF_SIZE[0] <= COLUMN + 0x52);
        assert!(half_place(3)[1] + HALF_SIZE[1] <= ROWS[0] + 0x2A);
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
