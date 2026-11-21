//! The controller screen's variables: the history of the polls the direction learner keeps,
//! the binding being learnt and the names the bindings are drawn with. A struct of plain cells,
//! one static.

use std::cell::Cell;

/// The controller screen's variables.
pub struct ControlsScreen {
    /// The left, right, y and z directions' histories of the
    /// last 32 polls (`screens::classify_controller`: a 1 for "under 0x80").
    pub history: [Cell<u32>; 4],
    /// 1 while a control learns (the keys are not looked at).
    pub learning: Cell<u32>,
    /// The clock when it began.
    pub learn_start: Cell<u32>,
    /// The control learning (from 1), -1 when it timed out.
    pub learned: Cell<u32>,
    /// The axis (0 ..= 5) the axes list opened the axis page for.
    pub axis: Cell<u32>,
    /// The page of controls the buttons page shows (0 or 1).
    pub page: Cell<u32>,
    /// The string ids of the directions of a pad (a device kind below 2).
    pub pad_names: [u16; 9],
    /// The string ids of the directions of a stick.
    pub stick_names: [u16; 9],
}

impl ControlsScreen {
    /// The pad's direction names.
    pub fn pad(&self) -> [u16; 9] {
        self.pad_names
    }

    /// The stick's direction names.
    pub fn stick(&self) -> [u16; 9] {
        self.stick_names
    }
}

impl ControlsScreen {
    pub const fn new() -> Self {
        ControlsScreen {
            history: [const { Cell::new(0xFFFF_FFFF) }; 4],
            learning: Cell::new(0),
            learn_start: Cell::new(0),
            learned: Cell::new(0),
            axis: Cell::new(0),
            page: Cell::new(0),
            pad_names: [0, 0x3C4, 0x3C3, 0, 0x3C1, 0, 0, 0, 0x3C2],
            stick_names: [0, 0x3B2, 0x3B1, 0, 0x3C1, 0, 0, 0, 0x3C2],
        }
    }
}
