//! The work car's state kept for a step and put back (converted 2026-10-21 from
//! `physics::save_car_state` 0x43DBD5 and `physics::restore_saved_car` 0x43DC25): its
//! place (`dword_71A10C` ..), its matrix (`g_crashMatrix`, 0x10 dwords) and its probe
//! points' faces (`dword_71ADC8` .., `dword_71A238` of them), with a counter of the puts
//! back (`dword_614738`). What the pieces are is a reading.

use crate::Invalid;

/// The most faces the kept list holds (`dword_6D2814` has 0x40 before `dword_6D2914`).
pub const MOST_SAVED: u32 = 0x40;
/// The most faces the work copy's list holds (0x71ADC8 to the copy's end, 0x71AFB8).
pub const MOST_LISTED: u32 = (0x1E0 + 0x10) / 4;
/// The matrix's dwords.
pub const MATRIX: usize = 0x10;

/// The pieces, as dwords.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CarState {
    pub place: [u32; 3],
    pub matrix: [u32; MATRIX],
    pub faces: Vec<u32>,
}

/// `physics::save_car_state`: the faces' count checked before anything (refused past
/// `MOST_SAVED`); the counter is then 0.
pub fn save_count(count: u32) -> Result<usize, Invalid> {
    if count > MOST_SAVED {
        return Err(Invalid(
            "a list of more than 0x40 dwords (past dword_6D2814's room)",
        ));
    }
    Ok(count as usize)
}

/// `physics::restore_saved_car`: the faces' count checked before anything (refused past
/// `MOST_LISTED`).
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
