//! The current car's work copy (from `drive::load_car` 0x460C40 and
//! `drive::store_car` 0x460C5D): the physics and the drawing work on one copy (`g_car`,
//! 0xFE0 bytes), copied in from the current car's block (`g_carBlocks[g_currentCar & 7]`)
//! and back out to it.

use crate::Invalid;

/// The copy's size in bytes.
pub const CAR_LEN: u32 = 0xFE0;

/// The current car's slot: its number's low three bits (the original's `and`).
pub fn car_slot(current: u32) -> u32 {
    current & 7
}

/// The slot's block, refused when null (the original's `rep movsd` would fault there).
pub fn car_block(block: u32) -> Result<u32, Invalid> {
    if block == 0 {
        return Err(Invalid("the current car's block is null"));
    }
    Ok(block)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks() {
        assert_eq!(car_slot(0x0F), 7);
        assert!(car_block(0).is_err());
        assert_eq!(car_block(0x1000), Ok(0x1000));
    }
}
