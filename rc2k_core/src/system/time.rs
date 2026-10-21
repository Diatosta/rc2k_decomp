//! The game's time stamp (converted 2026-10-21 from `system::packed_local_time` 0x410F99):
//! the local date and time to the minute in one dword: the year, then 4 bits of month, 5 of
//! day, 5 of hour, 6 of minute. The save files carry it. Also the start time kept
//! (`system::keep_start_time` 0x410F29) and the millisecond clock's override
//! (`system::tick_count` 0x411024), converted 2026-10-21.

/// The local time as Windows gives it (`SYSTEMTIME`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LocalTime {
    pub year: u16,
    pub month: u16,
    pub weekday: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

/// `system::keep_start_time`'s nine bytes (`byte_51839F` on, what the log's date and the
/// random seed read): weekday, day, month, the year as a word, hour, minute, second,
/// hundredths.
pub fn start_time(t: &LocalTime) -> [u8; 9] {
    let year = t.year.to_le_bytes();
    [
        t.weekday as u8,
        t.day as u8,
        t.month as u8,
        year[0],
        year[1],
        t.hour as u8,
        t.minute as u8,
        t.second as u8,
        (t.milliseconds as u32 / 10) as u8,
    ]
}

/// `system::tick_count`: Windows' clock, or with the override switch at 1 the override's
/// value (asked only then).
pub fn clock(now: u32, override_on: u32, value: impl FnOnce() -> u32) -> u32 {
    if override_on == 1 { value() } else { now }
}

pub fn pack(year: u16, month: u16, day: u16, hour: u16, minute: u16) -> u32 {
    let mut stamp = year as u32;
    for (bits, field) in [(4, month), (5, day), (5, hour), (6, minute)] {
        stamp = (stamp << bits) | field as u32;
    }
    stamp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_and_clock() {
        let t = LocalTime {
            year: 2026,
            month: 10,
            weekday: 3,
            day: 21,
            hour: 13,
            minute: 5,
            second: 59,
            milliseconds: 999,
        };
        assert_eq!(start_time(&t), [3, 21, 10, 0xEA, 0x07, 13, 5, 59, 99]);
        assert_eq!(clock(5, 1, || 7), 7);
        assert_eq!(clock(5, 2, || unreachable!()), 5);
    }

    #[test]
    fn pack() {
        assert_eq!(
            super::pack(2026, 10, 21, 13, 5),
            ((((2026 << 4 | 10) << 5 | 21) << 5 | 13) << 6) | 5
        );
    }
}
