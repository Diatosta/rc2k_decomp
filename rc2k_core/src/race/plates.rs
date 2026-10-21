//! The views' name plates: when the time plate is drawn again and how a car's name is
//! asked for (converted 2026-10-21 from `race::update_name_plates` 0x45B3B9).

/// `race::update_name_plates`: whether the two time texts (8 bytes each at
/// `dword_6D1838` / `dword_6D1844`, four dwords) differ from their last copies
/// (`dword_6D1850` ..), compared dword by dword as the original does.
pub fn times_changed(now: [u32; 4], kept: [u32; 4]) -> bool {
    now != kept
}

/// The ids below which a name is a string id (`race::update_name_plates`).
pub const STRING_IDS: u32 = 0x1_0000;

/// A name asked for at a car's +0x71C.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameAsk {
    /// None asked (0).
    Nothing,
    /// A string id (below 0x10000), drawn by `race::draw_plate_by_id`.
    Id(u32),
    /// A text's address, drawn by `race::draw_name_plate`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plates() {
        assert!(!times_changed([1, 2, 3, 4], [1, 2, 3, 4]));
        assert!(times_changed([1, 2, 3, 5], [1, 2, 3, 4]));
        assert_eq!(NameAsk::of(0), NameAsk::Nothing);
        assert_eq!(NameAsk::of(0xFFFF), NameAsk::Id(0xFFFF));
        assert_eq!(NameAsk::of(0x1_0000), NameAsk::Text(0x1_0000));
    }
}
