//! The menu loop's flags: the item highlighted, the leaving flags, the text entry's counters, the
//! sprite table and the menu shown. The game keeps these dwords at addresses one past a multiple
//! of four; here they lie at the same distances from the start. One static.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// The menu loop's flags. Plain fields behind one `RefCell`: the game's thread alone reads and
/// writes them, each borrow lasting one statement. The current menu is a handle of its own.
pub struct LoopFlags {
    values: GameThread<RefCell<LoopValues>>,
    /// The current menu (by handle, `menus_store::MenuId`).
    pub menu: crate::menu::menus_store::MenuCell,
}

/// The menu loop's values.
#[derive(Clone, Copy)]
pub struct LoopValues {
    /// The highlighted menu item, from 1.
    pub item: u32,
    /// True after a long time without a key (`menu::update_blink`).
    pub idle: bool,
    /// The main menu's frames without a key.
    pub idle_frames: u32,
    /// True when the event is left too, not only the screen (`go_back` clears it, confirmed
    /// leaving sets it; the loop leaves on it after an action, the flows read it).
    pub event_left: bool,
    /// True leaves the screen at once. Set by Escape, `go_back`, `leave_with_carry`; cleared by the
    /// key poll, set-up and the loop's action check.
    pub leave_screen: bool,
    /// Set with `leave_screen` by `leave_with_carry`; the class result pages set it while a page
    /// with the player is shown; a review ends the event when it is not set.
    pub left_with_carry: bool,
    /// An action asks to leave the screen (`ask_to_leave`): the loop leaves after the action, and a
    /// screen with `also_if` puts the leave question.
    pub ask_leave: bool,
    /// The characters typed so far (`menu::text_entry`).
    pub text_length: u32,
    /// The most characters the text takes.
    pub text_max: u32,
    /// Flipped by key 0x20 (`menu::handle_keys`).
    pub toggle: bool,
    /// The championship row highlighted.
    pub champ_row: u32,
    /// True once the options file has been read.
    pub options_loaded: bool,
    /// The menu sprites' glyph table.
    pub sprite_table: u32,
}

impl LoopFlags {
    pub const fn new() -> Self {
        LoopFlags {
            values: GameThread::new(RefCell::new(LoopValues {
                item: 0x1,
                idle: false,
                idle_frames: 0x0,
                event_left: false,
                leave_screen: false,
                left_with_carry: false,
                ask_leave: false,
                text_length: 0x0,
                text_max: 0x1E,
                toggle: true,
                champ_row: 0x0,
                options_loaded: false,
                sprite_table: 0x0,
            })),
            menu: crate::menu::menus_store::MenuCell::new(),
        }
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> LoopValues {
        *self.values.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, LoopValues> {
        self.values.get().borrow_mut()
    }
}
