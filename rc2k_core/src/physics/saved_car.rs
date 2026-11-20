//! The work car's state kept for a step and put back: its place, its matrix (0x10 dwords) and
//! its probe points' faces, with a counter of the puts back.

use crate::Invalid;

/// The most faces the kept list holds.
pub(crate) const MOST_SAVED: u32 = 0x40;
/// The most faces the work copy's list holds.
pub const MOST_LISTED: u32 = (0x1E0 + 0x10) / 4;
/// The matrix's dwords.
pub(crate) const MATRIX: usize = 0x10;

/// The pieces kept, as dwords.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CarState {
    pub place: [u32; 3],
    pub matrix: [u32; MATRIX],
    pub faces: Vec<u32>,
}

/// The faces' count, checked before anything (refused past `MOST_SAVED`); the counter is then 0.
pub fn save_count(count: u32) -> Result<usize, Invalid> {
    if count > MOST_SAVED {
        return Err(Invalid(
            "a list of more than 0x40 dwords (past the saved list's room)",
        ));
    }
    Ok(count as usize)
}

/// The faces' count, checked before anything (refused past `MOST_LISTED`).
pub fn restore_count(count: u32) -> Result<usize, Invalid> {
    if count > MOST_LISTED {
        return Err(Invalid(
            "a list of more than 0x7C dwords (past the work copy)",
        ));
    }
    Ok(count as usize)
}

/// The counter after a put back.
pub fn restored(counter: u32) -> u32 {
    counter.wrapping_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts() {
        assert_eq!(MOST_LISTED, 0x7C);
        assert_eq!(save_count(0x40), Ok(0x40));
        assert!(save_count(0x41).is_err());
        assert_eq!(restore_count(0x7C), Ok(0x7C));
        assert!(restore_count(0x7D).is_err());
        assert_eq!(restored(u32::MAX), 0);
    }
}
