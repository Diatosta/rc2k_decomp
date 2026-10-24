//! DOS file times (date << 16 | time, two-second steps), as `FileTimeToDosDateTime` and
//! `DosDateTimeToFileTime` make and read them: the game copies a file's last write time
//! through one (`files::copy_file`). Neither converts time zones: the DOS fields are the
//! UTC time's. Ticks are a Windows file time's: 100 ns since 1601-01-01 UTC.

/// Ticks from 1601-01-01 to 1980-01-01.
const TICKS_1980: u64 = 119_600_064_000_000_000;
const TICKS_PER_SECOND: u64 = 10_000_000;

fn leap(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn month_days(year: u32, month: u32) -> u32 {
    match month {
        2 if leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// `FileTimeToDosDateTime`: None before 1980 or past the last year DOS can hold (2107);
/// seconds rounded down to even.
pub fn from_ticks(ticks: u64) -> Option<u32> {
    let seconds = ticks.checked_sub(TICKS_1980)? / TICKS_PER_SECOND;
    let (mut days, rest) = ((seconds / 86_400) as u32, (seconds % 86_400) as u32);
    let mut year = 1980;
    loop {
        let len = if leap(year) { 366 } else { 365 };
        if days < len {
            break;
        }
        days -= len;
        year += 1;
    }
    if year > 2107 {
        return None;
    }
    let mut month = 1;
    while days >= month_days(year, month) {
        days -= month_days(year, month);
        month += 1;
    }
    let date = (year - 1980) << 9 | month << 5 | (days + 1);
    let time = (rest / 3600) << 11 | (rest / 60 % 60) << 5 | ((rest % 60) / 2);
    Some(date << 16 | time)
}

/// `DosDateTimeToFileTime`: None for fields that are not a time (month 0 or past 12, a
/// day past the month's, hour 24 or more, minute or second 60 or more).
pub fn to_ticks(dos: u32) -> Option<u64> {
    let (date, time) = (dos >> 16, dos & 0xFFFF);
    let (year, month, day) = ((date >> 9) + 1980, (date >> 5) & 0xF, date & 0x1F);
    let (hour, minute, second) = (time >> 11, (time >> 5) & 0x3F, (time & 0x1F) * 2);
    if !(1..=12).contains(&month)
        || day == 0
        || day > month_days(year, month)
        || hour >= 24
        || minute >= 60
        || second >= 60
    {
        return None;
    }
    let mut days = (1980..year)
        .map(|y| if leap(y) { 366 } else { 365 })
        .sum::<u64>();
    days += (1..month).map(|m| month_days(year, m) as u64).sum::<u64>() + (day - 1) as u64;
    let seconds = days * 86_400 + (hour * 3600 + minute * 60 + second) as u64;
    Some(TICKS_1980 + seconds * TICKS_PER_SECOND)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-10-24 13:45:37 UTC.
    const TICKS: u64 = 134_373_231_370_000_000;

    #[test]
    fn a_known_time() {
        let dos = from_ticks(TICKS).unwrap();
        assert_eq!(
            dos,
            (46 << 9 | 10 << 5 | 24) << 16 | (13 << 11 | 45 << 5 | 18)
        );
        // To two seconds: 37 s comes back as 36.
        assert_eq!(to_ticks(dos), Some(TICKS - TICKS_PER_SECOND));
    }

    #[test]
    fn the_edges() {
        assert_eq!(from_ticks(TICKS_1980), Some((1 << 5 | 1) << 16));
        assert_eq!(from_ticks(TICKS_1980 - 1), None);
        assert_eq!(to_ticks((1 << 5 | 1) << 16), Some(TICKS_1980));
        // 2107-12-31 23:59:58 is the last; 2108 is past.
        let last = (127 << 9 | 12 << 5 | 31) << 16 | (23 << 11 | 59 << 5 | 29);
        let ticks = to_ticks(last).unwrap();
        assert_eq!(from_ticks(ticks), Some(last));
        assert_eq!(from_ticks(ticks + 2 * TICKS_PER_SECOND), None);
    }

    #[test]
    fn fields_that_are_not_a_time() {
        let day = |y: u32, m: u32, d: u32| (y - 1980) << 9 | m << 5 | d;
        assert_eq!(to_ticks(day(2000, 0, 1) << 16), None);
        assert_eq!(to_ticks(day(2000, 13, 1) << 16), None);
        assert_eq!(to_ticks(day(2000, 1, 0) << 16), None);
        assert_eq!(to_ticks(day(2001, 2, 29) << 16), None);
        assert!(to_ticks(day(2000, 2, 29) << 16).is_some());
        assert_eq!(to_ticks(day(2000, 1, 1) << 16 | 24 << 11), None);
        assert_eq!(to_ticks(day(2000, 1, 1) << 16 | 60 << 5), None);
        assert_eq!(to_ticks(day(2000, 1, 1) << 16 | 30), None);
    }

    #[test]
    fn every_day_goes_round() {
        let mut ticks = TICKS_1980;
        while let Some(dos) = from_ticks(ticks) {
            assert_eq!(to_ticks(dos), Some(ticks));
            ticks += 86_400 * TICKS_PER_SECOND + 2 * TICKS_PER_SECOND * 1801;
        }
    }
}
