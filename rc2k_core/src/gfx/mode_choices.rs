//! The graphics options' screen mode choices.
//!
//! The choices: a pixel kind (0 or 1), a size (a place in a list of the mode table's modes
//! of that kind), a third choice (0 or 1) and a texture figure. The list is made again
//! from the mode table; the size's place follows the mode in use, then the race's mode.
//! A choice is tried as a mode: 0x300000 with the kind's bits, the list's mode and the
//! third choice's bits.

/// The most modes the list holds.
pub const ROOM: u32 = 64;
/// The limit on a size's place taken; a place past the list reads 16 x the place beyond it
/// (kept).
pub const PLACE_LIMIT: u32 = 0x1000;
/// What every mode tried has.
pub const TRIED: u32 = 0x30_0000;

/// The pixel kind's choice from the race's mode: 0 when its kind nibble (0xF000) is the
/// first kind's bits, else 1.
pub fn kind_of_race(race_mode: u32, first_kind_bits: u32) -> u32 {
    ((race_mode & 0xF000) != first_kind_bits) as u32
}

/// Whether the choices can be tried: kind and third 0 or 1, the size's place in the list.
pub fn can_try(kind: u32, place: u32, count: u32, third: u32) -> bool {
    kind <= 1 && place < count && third <= 1
}

/// The mode tried for the choices.
pub fn tried_mode(kind_bits: u32, list_mode: u32, third_bits: u32) -> u32 {
    TRIED | kind_bits | list_mode | third_bits
}

/// Whether a mode that works is kept for the race: its texture figure not above the
/// card's (unsigned).
pub fn kept(figure: u32, card_figure: u32) -> bool {
    figure <= card_figure
}

/// A mode in the list: its size, its pixel kind's figure and its bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Listed {
    pub width: u32,
    pub height: u32,
    pub figure: u32,
    pub mode: u32,
}

/// The size key and pixel kind key of a mode (its tables' first dwords).
pub fn keys(mode: u32) -> (u32, u32) {
    (mode & 0xFF, mode & 0xF00)
}

/// The third choice's key in its table.
pub fn third_key(race_mode: u32) -> u32 {
    race_mode & 0xF_0000
}

/// The bits that tell one listed mode from another: the size and pixel kind (the low 12 bits) and
/// the number of a size beyond the game's table (the top byte, see `CUSTOM_SHIFT`).
pub const SAME_SIZE: u32 = 0xFF00_0FFF;
/// Where a mode with a size beyond the game's table holds that size's number (from 1).
pub const CUSTOM_SHIFT: u32 = 24;

/// The size's place of the mode in use (`current`, by its size bits; 0 when none matches). The
/// caller stores it and then `race_place`'s result over it.
pub fn in_use_place(modes: &[u32], current: u32) -> u32 {
    modes
        .iter()
        .position(|&m| (m ^ current) & SAME_SIZE == 0)
        .unwrap_or(0) as u32
}

/// The size's place and third choice of the race's mode (by its low 12 bits), the third
/// from its table (`third`, asked only when found: the entry's value less 1, not below 0);
/// not found, both 0.
pub fn race_place(modes: &[u32], race_mode: u32, third: impl FnOnce() -> u32) -> (u32, u32) {
    match modes
        .iter()
        .position(|&m| m & SAME_SIZE == race_mode & SAME_SIZE)
    {
        Some(n) => (n as u32, (third() as i32).wrapping_sub(1).max(0) as u32),
        None => (0, 0),
    }
}

/// The texture figure kept: the choice masked by the card's.
pub fn figure(choice: u32, card_figure: u32) -> u32 {
    choice & card_figure
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choices() {
        assert_eq!(kind_of_race(0x2803, 0x1000), 1);
        assert_eq!(kind_of_race(0x1803, 0x1000), 0);
        assert!(can_try(1, 2, 3, 0) && !can_try(2, 0, 3, 0) && !can_try(0, 3, 3, 0));
        assert!(!can_try(0, 0, 3, 2));
        assert_eq!(tried_mode(0x800, 0x1203, 0x1_0000), 0x31_1A03);
        assert!(kept(3, 3) && !kept(4, 3));
        assert_eq!(keys(0x1_0A05), (5, 0xA00));
        assert_eq!(third_key(0x2_0A05), 0x2_0000);
        let modes = [0x1801, 0x1802, 0x1803];
        assert_eq!(in_use_place(&modes, 0xF_1802), 1);
        assert_eq!(in_use_place(&modes, 0x1804), 0);
        assert_eq!(race_place(&modes, 0x2_1803, || 2), (2, 1));
        assert_eq!(race_place(&modes, 0x2_1803, || 0), (2, 0));
        assert_eq!(race_place(&modes, 0x1804, || unreachable!()), (0, 0));
        assert_eq!(figure(0xF, 0x6), 6);
    }
}
