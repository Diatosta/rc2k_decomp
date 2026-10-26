//! A computer car put at a start on the line (converted 2026-10-21 from
//! `drive::start_car_on_line` 0x437613; the motion's turn stays the hook's): which
//! checkpoint it starts at and its share between two figures by its start's number.
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores: the same bits at the game's double precision.

use crate::Invalid;

/// The highest start number worked out (`dword_6051A0` indexes `dword_605034`).
pub const LAST_START: u32 = 0x400;

/// The start number checked.
pub fn check_start(n: u32) -> Result<(), Invalid> {
    if n > LAST_START {
        return Err(Invalid("a start number above 0x400 (not worked out)"));
    }
    Ok(())
}

/// The grades' mode a started car takes (`dword_71A234`).
pub const STARTED_MODE: u32 = 2;

/// The car's checkpoint: the lead car's (`dword_70F484`) plus the checkpoints on (ebx).
pub fn start_checkpoint(lead: u32, ahead: u32) -> u32 {
    lead.wrapping_add(ahead)
}

/// The share (`flt_6051AC`, `flt_71A650`) of start n of m between lo and hi:
/// lo + ((m - 1) - n) / (m - 1) x (hi - lo), "1" the float `one` (`flt_5C5D38`), in the
/// original's order (`fild n; fild m; fsub one; fsubr; fdivp; fld hi; fsub lo; fmulp;
/// fadd lo`).
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
