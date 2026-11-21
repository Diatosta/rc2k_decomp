//! The run-time state every list menu shares, whichever screen shows the list (the installer's
//! pages, the network lobby and session lists, the replay viewer): the highlighted item, the key
//! being repeated, the answer and the clocks that pace the highlight. One static.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// The list menu in use. Plain fields behind one `RefCell`: the game's thread alone reads and
/// writes them, each borrow lasting one statement.
pub struct ListState(GameThread<RefCell<ListValues>>);

/// The values the list menus share.
#[derive(Clone, Copy)]
pub struct ListValues {
    /// The highlighted item.
    pub choice: u32,
    /// The highlighted item before the last key.
    pub choice_before: u32,
    /// How many times the held key has repeated.
    pub repeats: u32,
    /// The last key taken (the replay viewer reads it too).
    pub last_key: u8,
    /// A key was taken this frame.
    pub key_taken: u8,
    /// The answer the list leaves.
    pub list_answer: u8,
    /// The list is being left.
    pub list_leave: u8,
    /// The clock of the last key.
    pub key_clock: u32,
    /// The clock of the frame being drawn.
    pub frame_clock: u32,
    /// The clock of the last move of the highlight.
    pub move_clock: u32,
}

impl ListState {
    pub const fn new() -> Self {
        ListState(GameThread::new(RefCell::new(ListValues {
            choice: 0,
            choice_before: 0,
            repeats: 1,
            last_key: 0,
            key_taken: 0,
            list_answer: 0,
            list_leave: 0,
            key_clock: 0,
            frame_clock: 0,
            move_clock: 0,
        })))
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> ListValues {
        *self.0.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, ListValues> {
        self.0.get().borrow_mut()
    }
}
