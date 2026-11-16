//! The controller screen's variables: the history of the polls the direction learner keeps,
//! the binding being learnt and the names the bindings are drawn with. A struct of plain cells,
//! one static.

use crate::memory::session::{PlainDword, PlainWord};

/// The controller screen's variables.
#[derive(rc2k_derive::Cells)]
pub struct ControlsScreen {
    /// The left, right, y and z directions' histories of the
    /// last 32 polls (`screens::classify_controller`: a 1 for "under 0x80").
    pub history: [PlainDword; 4],
    /// 1 while a control learns (the keys are not looked at).
    pub learning: PlainDword,
    /// The clock when it began.
    pub learn_start: PlainDword,
    /// The control learning (from 1), -1 when it timed out.
    pub learned: PlainDword,
    /// The axis (0 ..= 5) the axes list opened the axis page for.
    pub axis: PlainDword,
    /// The page of controls the buttons page shows (0 or 1).
    pub page: PlainDword,
    /// The string ids of the directions of a pad (a device kind below 2).
    pub pad_names: [PlainWord; 9],
    /// The string ids of the directions of a stick.
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

impl ControlsScreen {
    pub const fn new() -> Self {
        ControlsScreen {
            history: [const { PlainDword::new(0xFFFF_FFFF) }; 4],
            learning: PlainDword::new(0),
            learn_start: PlainDword::new(0),
            learned: PlainDword::new(0),
            axis: PlainDword::new(0),
            page: PlainDword::new(0),
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
