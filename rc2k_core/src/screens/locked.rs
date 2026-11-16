//! The notes a menu puts up for a choice that is not open yet: a message box, unless a cheat
//! opens everything.

/// The cheat bits that open item 3 (bit 0x11) and the stages (bit 0).
pub const ITEM_3_CHEAT: u32 = 0x11;
pub const STAGES_CHEAT: u32 = 0;

/// What a note step decides: whether the box is put up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Note {
    pub show: bool,
}

fn cheat(cheats: u32, bit: u32) -> bool {
    cheats & (1 << bit) != 0
}

/// `note_locked_item_3`: with item 3 chosen and its switch not 1.
pub fn item_3(cheats: u32, item: u32, switch: u32) -> Note {
    if cheat(cheats, ITEM_3_CHEAT) {
        return Note { show: false };
    }
    Note {
        show: item == 3 && switch != 1,
    }
}

/// `note_locked_stage`: items 1 and 7 are not stages; item n's stage is open when
/// `open(n - 2)` (`Arcade.levels_won`, wrapping) is 1.
pub fn stage(cheats: u32, item: u32, open: impl FnOnce(u32) -> u8) -> Note {
    if cheat(cheats, STAGES_CHEAT) {
        return Note { show: false };
    }
    if item == 1 || item == 7 {
        return Note { show: false };
    }
    let index = item.wrapping_sub(2);
    Note {
        show: open(index) != 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes() {
        assert_eq!(item_3(1 << 0x11, 3, 0), Note { show: false });
        assert_eq!(item_3(0, 3, 0), Note { show: true });
        assert!(!item_3(0, 3, 1).show);
        assert!(!item_3(0, 2, 0).show);
        assert_eq!(stage(1, 4, |_| 0), Note { show: false });
        assert_eq!(stage(0, 7, |_| 0), Note { show: false });
        assert_eq!(stage(0, 4, |i| (i == 2) as u8), Note { show: false });
        assert_eq!(stage(0, 5, |i| (i == 2) as u8), Note { show: true });
    }
}
