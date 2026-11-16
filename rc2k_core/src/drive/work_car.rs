//! The current car's work copy: the physics and the drawing work on one copy, copied in from the
//! current car's block and back out to it.

use crate::Invalid;

/// The copy's size in bytes.
pub const CAR_LEN: u32 = 0xFE0;

/// The current car's slot: its number's low three bits.
pub fn car_slot(current: u32) -> u32 {
    current & 7
}

/// The slot's block. Refused: a null block (the copy would fault).
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
