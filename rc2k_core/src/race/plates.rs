//! The views' name plates: when the time plate redraws, how a car's name is asked for and the
//! plate layouts.

/// Whether the two time texts (8 bytes each) differ from their last copies.
pub fn times_changed(now: [u32; 4], kept: [u32; 4]) -> bool {
    now != kept
}

/// Ids below this are string ids; a name at or above it is a text.
pub const STRING_IDS: u32 = 0x1_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameAsk {
    /// None asked (0).
    Nothing,
    /// A string id (below `STRING_IDS`).
    Id(u32),
    /// A text's address.
    Text(u32),
}

impl NameAsk {
    pub fn of(word: u32) -> Self {
        match word {
            0 => Self::Nothing,
            w if w < STRING_IDS => Self::Id(w),
            w => Self::Text(w),
        }
    }
}

/// The name font's box width on a name plate.
pub const NAME_BOX: u32 = 0x168;
/// The time font's box width on a time plate.
pub const TIME_BOX: u32 = 0x80;

/// A name plate: three lines cleared, the name drawn three times (a long name wraps) at x 4,
/// -0x74, -0xEC and y 0, a line, two lines.
pub fn name_rows(line: u32) -> u32 {
    line.wrapping_mul(3)
}

pub fn name_places(line: u32) -> [(u32, u32); 3] {
    let (mut x, mut y) = (4u32, 0u32);
    std::array::from_fn(|n| {
        if n > 0 {
            x = x.wrapping_sub(0x78);
            y = y.wrapping_add(line);
        }
        (x, y)
    })
}

/// A time plate below the name's three lines: first row's byte offset, rows cleared, and the
/// two times' places (x 4, y three and four lines).
pub fn time_start(line: u32, pitch: u32) -> u32 {
    (line.wrapping_mul(3) as u64 * pitch as u64) as u32
}

pub fn time_rows(line: u32) -> u32 {
    line.wrapping_mul(2).wrapping_add(3)
}

pub fn time_places(line: u32) -> [(u32, u32); 2] {
    std::array::from_fn(|n| {
        (
            4,
            line.wrapping_mul(3)
                .wrapping_add(line.wrapping_mul(n as u32)),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts() {
        assert_eq!(name_rows(0x10), 0x30);
        assert_eq!(
            name_places(0x10),
            [
                (4, 0),
                (4u32.wrapping_sub(0x78), 0x10),
                (4u32.wrapping_sub(0xF0), 0x20)
            ]
        );
        assert_eq!(time_start(0x10, 0x200), 0x6000);
        assert_eq!(time_start(0x10, 0x8000_0000), 0);
        assert_eq!(time_rows(0x10), 0x23);
        assert_eq!(time_places(0x10), [(4, 0x30), (4, 0x40)]);
    }

    #[test]
    fn plates() {
        assert!(!times_changed([1, 2, 3, 4], [1, 2, 3, 4]));
        assert!(times_changed([1, 2, 3, 5], [1, 2, 3, 4]));
        assert_eq!(NameAsk::of(0), NameAsk::Nothing);
        assert_eq!(NameAsk::of(0xFFFF), NameAsk::Id(0xFFFF));
        assert_eq!(NameAsk::of(0x1_0000), NameAsk::Text(0x1_0000));
    }
}
