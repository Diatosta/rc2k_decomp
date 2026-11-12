//! The controller screen's variables, out of RAL.EXE's image (IDA 0x755170 .. 0x755224): the history of the polls the direction learner keeps, the binding being
//! learnt, and the names the bindings are drawn with. A `#[repr(C)]` struct of plain cells
//! with the image's layout, one static. (It starts with the unreferenced
//! bytes after the savegame preview's figures, `saves::preview_store`.)

use crate::session::{PlainBytes, PlainDword, PlainWord};

/// The controller screen's variables (IDA 0x755170 .. 0x755224).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ControlsScreen {
    /// Unreferenced bytes (IDA 0X755170 .. 0X755178).
    pub _gap_755170: PlainBytes<8>,
    /// Unreferenced (IDA 0X755178): 1 .. 12 and a -1.
    pub _table_755178: [PlainDword; 13],
    /// Unreferenced bytes (IDA 0X7551AC .. 0X7551E0).
    pub _gap_7551ac: PlainBytes<52>,
    /// Unreferenced (IDA 0X7551E0): -1.
    pub _unused_7551e0: PlainDword,
    /// `dword_7551E4` .. `dword_7551F0`: the left, right, y and z directions' histories of the
    /// last 32 polls (`screens::classify_controller`: a 1 for "under 0x80").
    pub history: [PlainDword; 4],
    /// `g_bindingLearning`: 1 while a control learns (the keys are not looked at).
    pub learning: PlainDword,
    /// `g_bindingLearnStart`: the clock when it began.
    pub learn_start: PlainDword,
    /// `g_bindingLearned`: the control learning (from 1), -1 when it timed out.
    pub learned: PlainDword,
    /// `word_755200`: the string ids of the directions of a pad (a device kind below 2).
    pub pad_names: [PlainWord; 9],
    /// `word_755212`: the string ids of the directions of a stick.
    pub stick_names: [PlainWord; 9],
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
            _gap_755170: PlainBytes::new([0; 8]),
            _table_755178: [
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(3),
                PlainDword::new(4),
                PlainDword::new(5),
                PlainDword::new(6),
                PlainDword::new(7),
                PlainDword::new(8),
                PlainDword::new(9),
                PlainDword::new(10),
                PlainDword::new(11),
                PlainDword::new(12),
                PlainDword::new(0xFFFF_FFFF),
            ],
            _gap_7551ac: PlainBytes::new([0; 52]),
            _unused_7551e0: PlainDword::new(0xFFFF_FFFF),
            history: [const { PlainDword::new(0xFFFF_FFFF) }; 4],
            learning: PlainDword::new(0),
            learn_start: PlainDword::new(0),
            learned: PlainDword::new(0),
            pad_names: [
                PlainWord::new(0),
                PlainWord::new(0x3C4),
                PlainWord::new(0x3C3),
                PlainWord::new(0),
                PlainWord::new(0x3C1),
                PlainWord::new(0),
                PlainWord::new(0),
                PlainWord::new(0),
                PlainWord::new(0x3C2),
            ],
            stick_names: [
                PlainWord::new(0),
                PlainWord::new(0x3B2),
                PlainWord::new(0x3B1),
                PlainWord::new(0),
                PlainWord::new(0x3C1),
                PlainWord::new(0),
                PlainWord::new(0),
                PlainWord::new(0),
                PlainWord::new(0x3C2),
            ],
        }
    }
}

/// IDA 0X755170, the first byte of the piece.
#[cfg(test)]
pub const START: usize = 0x755170;
/// IDA 0X755224, the end of the piece.
#[cfg(test)]
pub const END: usize = 0x755224;

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn controls_screen_starts_as_the_games() {
        assert!(END - START <= size_of::<ControlsScreen>());
        assert!(size_of::<ControlsScreen>() <= (END - START).next_multiple_of(4));
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::screens::owner::SCREENS_OWNER.get().controls,
        );
        assert_eq!(offset_of!(ControlsScreen, _table_755178), 0x8);
        assert_eq!(offset_of!(ControlsScreen, history), 0x74);
        assert_eq!(offset_of!(ControlsScreen, learning), 0x84);
        assert_eq!(offset_of!(ControlsScreen, pad_names), 0x90);
        assert_eq!(offset_of!(ControlsScreen, stick_names), 0xa2);
    }
}
