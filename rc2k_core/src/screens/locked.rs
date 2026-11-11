//! The notes a menu puts up for a choice that is not open yet (from
//! `screens::note_locked_item_3` 0x4AAAB8 and `note_locked_stage` 0x48ADCE): a message box,
//! unless a cheat opens everything.

/// The cheat bit that opens item 3 (`g_cheats`, bit 0x11), and the one that opens the stages
/// (bit 0).
pub const ITEM_3_CHEAT: u32 = 0x11;
pub const STAGES_CHEAT: u32 = 0;

/// What a note step leaves: eax as the original leaves it (`None`: not touched) and whether
/// the box is put up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Note {
    pub eax: Option<u32>,
    pub show: bool,
}

fn cheat(cheats: u32, bit: u32) -> bool {
    cheats & (1 << bit) != 0
}

/// `note_locked_item_3`: with item 3 chosen and its switch (`dword_75ACCC`) not 1.
pub fn item_3(cheats: u32, item: u32, switch: u32) -> Note {
    if cheat(cheats, ITEM_3_CHEAT) {
        return Note {
            eax: None,
            show: false,
        };
    }
    Note {
        eax: Some(item),
        show: item == 3 && switch != 1,
    }
}

/// `note_locked_stage`: items 1 and 7 are not stages; item n's stage is open when
/// `open(n - 2)` (`byte_758800`, wrapping as the original's address does) is 1.
pub fn stage(cheats: u32, item: u32, open: impl FnOnce(u32) -> u8) -> Note {
    if cheat(cheats, STAGES_CHEAT) {
        return Note {
            eax: None,
            show: false,
        };
    }
    if item == 1 || item == 7 {
        return Note {
            eax: Some(item),
            show: false,
        };
    }
    let index = item.wrapping_sub(2);
    Note {
        eax: Some(index),
        show: open(index) != 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes() {
        assert_eq!(
            item_3(1 << 0x11, 3, 0),
            Note {
                eax: None,
                show: false
            }
        );
        assert_eq!(
            item_3(0, 3, 0),
            Note {
                eax: Some(3),
                show: true
            }
        );
        assert!(!item_3(0, 3, 1).show);
        assert!(!item_3(0, 2, 0).show);
        assert_eq!(
            stage(1, 4, |_| 0),
            Note {
                eax: None,
                show: false
            }
        );
        assert_eq!(
            stage(0, 7, |_| 0),
            Note {
                eax: Some(7),
                show: false
            }
        );
        assert_eq!(
            stage(0, 4, |i| (i == 2) as u8),
            Note {
                eax: Some(2),
                show: false
            }
        );
        assert_eq!(
            stage(0, 5, |i| (i == 2) as u8),
            Note {
                eax: Some(3),
                show: true
            }
        );
    }
}
