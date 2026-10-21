//! The number plates' textures (converted 2026-10-21 from `scene::load_number_plates`
//! 0x4255AA): two plates, the player's car's and the other's, each by its kind's plate
//! number: one of two pairs of pictures and two numbers from the plates' tables, uploaded
//! at one of two rows of the texture.

use crate::Invalid;

/// The plates' tables' room (`dword_5DCD9C`, `dword_5DCE2C`: 0x90 bytes each).
pub const NUMBERS: u32 = 0x24;
/// Plate numbers below this take the first pair of pictures (`tx1822`, `tx1823`), the
/// rest the second (`tx1824`, `tx1825`).
pub const FIRST_PAIR_BELOW: u32 = 0x12;
/// The two plates' rows (ebp) and their column (edi).
pub const ROWS: [u32; 2] = [0x68, 0x94];
pub const COLUMN: u32 = 0xAD;
/// The most cars (`g_carBlocks`).
pub const CARS: u32 = 8;

/// Refuses a plate number past the tables, before anything is uploaded.
pub fn check_numbers(numbers: [u32; 2]) -> Result<(), Invalid> {
    if numbers.iter().any(|&n| n >= NUMBERS) {
        return Err(Invalid("a plate's number past its tables"));
    }
    Ok(())
}

/// The first of a plate's pair of pictures (0 or 2 of the four names).
pub fn pictures(number: u32) -> usize {
    if number < FIRST_PAIR_BELOW { 0 } else { 2 }
}

/// The one car shown (`dword_52A920`): its block's number; refuses one past the blocks.
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
    fn plates() {
        assert!(check_numbers([0, 0x24]).is_err());
        assert!(check_numbers([0x23, 0]).is_ok());
        assert_eq!((pictures(0x11), pictures(0x12)), (0, 2));
        assert!(one_car(8).is_err());
        assert_eq!(one_car(7), Ok(7));
    }
}
