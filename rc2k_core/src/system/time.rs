//! The game's time stamp (converted 2026-10-21 from `system::packed_local_time` 0x410F99):
//! the local date and time to the minute in one dword: the year, then 4 bits of month, 5 of
//! day, 5 of hour, 6 of minute. The save files carry it.

pub fn pack(year: u16, month: u16, day: u16, hour: u16, minute: u16) -> u32 {
    let mut stamp = year as u32;
    for (bits, field) in [(4, month), (5, day), (5, hour), (6, minute)] {
        stamp = (stamp << bits) | field as u32;
    }
    stamp
}

#[cfg(test)]
mod tests {
    #[test]
    fn pack() {
        assert_eq!(
            super::pack(2026, 10, 21, 13, 5),
            ((((2026 << 4 | 10) << 5 | 21) << 5 | 13) << 6) | 5
        );
    }
}
