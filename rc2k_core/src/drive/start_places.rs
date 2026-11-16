//! A computer car put at a start on the line: which checkpoint it starts at and its share between
//! two figures by its start's number. The motion's turn stays with the caller.
//!
//! Floats: sums are `f64` and rounded to `f32` where stored, the same bits as the game's x87 at
//! double precision.

use crate::Invalid;

/// The highest start number worked out (`RaceTables.start_slot` indexes
/// `RaceTables.arcade_car_numbers[0]`).
pub const LAST_START: u32 = 0x400;

/// Refused: a start number above `LAST_START`.
pub fn check_start(n: u32) -> Result<(), Invalid> {
    if n > LAST_START {
        return Err(Invalid(
            "a start number above 0x400 (the table holds 90; the original's check lets 0x400 through)",
        ));
    }
    Ok(())
}

/// The grades' mode a started car takes.
pub const STARTED_MODE: u32 = 2;

/// The car's checkpoint: the lead car's plus the checkpoints on.
pub fn start_checkpoint(lead: u32, ahead: u32) -> u32 {
    lead.wrapping_add(ahead)
}

/// The share of start n of m between lo and hi: lo + ((m - 1) - n) / (m - 1) x (hi - lo), "1"
/// the float `one`. The operations run in this order to give the game's bits.
pub fn start_share(n: i32, m: i32, one: f32, hi: f32, lo: f32) -> f32 {
    let m1 = f64::from(m) - f64::from(one);
    let share = (m1 - f64::from(n)) / m1;
    ((f64::from(hi) - f64::from(lo)) * share + f64::from(lo)) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts() {
        assert!(check_start(0x400).is_ok() && check_start(0x401).is_err());
        assert_eq!(start_checkpoint(10, 5), 15);
        assert_eq!(start_share(0, 3, 1.0, 4.0, 2.0), 4.0);
        assert_eq!(start_share(2, 3, 1.0, 4.0, 2.0), 2.0);
        assert_eq!(start_share(1, 3, 1.0, 4.0, 2.0), 3.0);
        assert!(start_share(0, 1, 1.0, 4.0, 2.0).is_nan());
    }
}
