//! The controller screen's variables, out of RAL.EXE's image (IDA 0x755170 .. 0x755224;
//! phase 7, typed): the history of the polls the direction learner keeps, the binding being
//! learnt, and the names the bindings are drawn with. A `#[repr(C)]` struct of atomic cells
//! with the image's layout, one static; `moved::MOVED` points the range at it. (Split
//! 2026-10-25 from the front end's block `menu::store`; it starts with the unreferenced
//! bytes after the savegame preview's figures, `saves::preview_store`.)

use crate::session::{Bytes, Dword, Word};

/// The controller screen's variables (IDA 0x755170 .. 0x755224).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ControlsScreen {
    /// Unreferenced bytes (IDA 0X755170 .. 0X755178).
    pub _gap_755170: Bytes<8>,
    /// Unreferenced (IDA 0X755178): 1 .. 12 and a -1.
    pub _table_755178: [Dword; 13],
    /// Unreferenced bytes (IDA 0X7551AC .. 0X7551E0).
    pub _gap_7551ac: Bytes<52>,
    /// Unreferenced (IDA 0X7551E0): -1.
    pub _unused_7551e0: Dword,
    /// `dword_7551E4` .. `dword_7551F0`: the left, right, y and z directions' histories of the
    /// last 32 polls (`screens::classify_controller`: a 1 for "under 0x80").
    pub history: [Dword; 4],
    /// `g_bindingLearning`: 1 while a control learns (the keys are not looked at).
    pub learning: Dword,
    /// `g_bindingLearnStart`: the clock when it began.
    pub learn_start: Dword,
    /// `g_bindingLearned`: the control learning (from 1), -1 when it timed out.
    pub learned: Dword,
    /// `word_755200`: the string ids of the directions of a pad (a device kind below 2).
    pub pad_names: [Word; 9],
    /// `word_755212`: the string ids of the directions of a stick.
    pub stick_names: [Word; 9],
}

impl ControlsScreen {
    /// The pad's direction names.
    pub fn pad(&self) -> [u16; 9] {
        std::array::from_fn(|n| self.pad_names[n].get())
    }

    /// The stick's direction names.
    pub fn stick(&self) -> [u16; 9] {
        std::array::from_fn(|n| self.stick_names[n].get())
    }
}

/// First values the image's (`tests::controls_screen_starts_as_the_games`).
impl ControlsScreen {
    pub const fn new() -> Self {
        ControlsScreen {
            _gap_755170: Bytes::new([0; 8]),
            _table_755178: [
                Dword::new(1),
                Dword::new(2),
                Dword::new(3),
                Dword::new(4),
                Dword::new(5),
                Dword::new(6),
                Dword::new(7),
                Dword::new(8),
                Dword::new(9),
                Dword::new(10),
                Dword::new(11),
                Dword::new(12),
                Dword::new(0xFFFF_FFFF),
            ],
            _gap_7551ac: Bytes::new([0; 52]),
            _unused_7551e0: Dword::new(0xFFFF_FFFF),
            history: [const { Dword::new(0xFFFF_FFFF) }; 4],
            learning: Dword::new(0),
            learn_start: Dword::new(0),
            learned: Dword::new(0),
            pad_names: [
                Word::new(0),
                Word::new(0x3C4),
                Word::new(0x3C3),
                Word::new(0),
                Word::new(0x3C1),
                Word::new(0),
                Word::new(0),
                Word::new(0),
                Word::new(0x3C2),
            ],
            stick_names: [
                Word::new(0),
                Word::new(0x3B2),
                Word::new(0x3B1),
                Word::new(0),
                Word::new(0x3C1),
                Word::new(0),
                Word::new(0),
                Word::new(0),
                Word::new(0x3C2),
            ],
        }
    }
}

/// IDA 0X755170, the first byte of the piece.
pub const START: usize = 0x755170;
/// IDA 0X755224, the end of the piece.
pub const END: usize = 0x755224;

/// Where IDA 0X755170 is now (`moved::MOVED`'s home).
#[cfg(test)]
pub fn controls_screen_addr() -> u32 {
    crate::pointers::game_address(&raw const crate::screens::owner::SCREENS_OWNER.controls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn controls_screen_starts_as_the_games() {
        assert!(END - START <= size_of::<ControlsScreen>());
        assert!(size_of::<ControlsScreen>() <= (END - START).next_multiple_of(4));
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(controls_screen_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(ControlsScreen, _table_755178), 0x755178);
        at(offset_of!(ControlsScreen, history), 0x7551E4);
        at(offset_of!(ControlsScreen, learning), 0x7551F4);
        at(offset_of!(ControlsScreen, pad_names), 0x755200);
        at(offset_of!(ControlsScreen, stick_names), 0x755212);
    }
}
