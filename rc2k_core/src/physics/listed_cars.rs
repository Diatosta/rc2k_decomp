//! The cars listed for a device or sound slot (converted 2026-10-21 from the three
//! twins `physics::set_flag_51659c` 0x45D151, `set_flag_5165a0` 0x45D16D and
//! `set_flag_5165a4` 0x45D189): a flag set when the current car has an entry in
//! `dword_641680` (eight, by car; -1 none).

use crate::Invalid;

/// No entry.
pub const NOT_LISTED: u32 = u32::MAX;

/// The current car's entry, `entry(car)` asked once the car is known to be one of the
/// eight (past 7 is refused: the table's size is not worked out); and whether the flag
/// is set by it (the entry not -1).
pub fn listed_entry(car: u32, entry: impl FnOnce(u32) -> u32) -> Result<(u32, bool), Invalid> {
    if car > 7 {
        return Err(Invalid("a current car past 7"));
    }
    let e = entry(car);
    Ok((e, e != NOT_LISTED))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries() {
        assert_eq!(listed_entry(3, |c| c * 2), Ok((6, true)));
        assert_eq!(listed_entry(7, |_| NOT_LISTED), Ok((NOT_LISTED, false)));
        assert!(listed_entry(8, |_| unreachable!()).is_err());
    }
}
