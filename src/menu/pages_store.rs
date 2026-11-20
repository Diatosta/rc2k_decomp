//! The front end's list pages and its yes / no answer: the pages the save, scores, set-up and
//! replay lists are on and the answer the yes / no screens leave.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// The list pages and the answer. Plain values behind one `RefCell`: the game's thread alone
/// reads and writes them, each borrow lasting one statement.
pub struct MenuPages(GameThread<RefCell<PageValues>>);

/// The pages' values.
#[derive(Clone, Copy)]
pub struct PageValues {
    /// The page the list of saved games is on (`menu::item_actions`, `menu::name_lists`).
    pub save_page: u32,
    /// The page of the best scores.
    pub scores_page: u32,
    /// The page of the set-up slots (`menu::load_setup_slot`).
    pub setup_slot_page: u32,
    /// The page of the replays (`menu::name_lists`, `menu::player_actions`).
    pub replay_page: u32,
    /// The answer of a yes / no screen: 1 = yes (at start), 0 = no.
    pub yes_no_answer: u32,
}

impl MenuPages {
    pub const fn new() -> Self {
        MenuPages(GameThread::new(RefCell::new(PageValues {
            save_page: 0x0,
            scores_page: 0x0,
            setup_slot_page: 0x0,
            replay_page: 0x0,
            yes_no_answer: 0x1,
        })))
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> PageValues {
        *self.0.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, PageValues> {
        self.0.get().borrow_mut()
    }
}
