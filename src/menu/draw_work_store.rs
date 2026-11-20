//! What the screens' drawing and loops hand on between frames and screens: the blink phases, the
//! item action's outcome, the Escape flag, and three carries (the plain buttons' last place and
//! the leaving value). One static per `Screens`.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// The drawing work variables. Plain fields behind one `RefCell`: the game's thread alone reads
/// and writes them, each borrow lasting one statement.
pub struct DrawWork(GameThread<RefCell<DrawValues>>);

/// The drawing work's values.
#[derive(Clone, Copy)]
pub struct DrawValues {
    /// True in the visible half of a blink (`menu::update_blink`).
    pub blink: bool,
    /// True in the visible half of the text cursor's blink.
    pub cursor_on: bool,
    /// The x a plain button's drawing ends at. The buttons that borrow their place start from it
    /// (`screens::buttons`), and the best-times fill leaves its sum here (`saves::default_times`).
    pub button_x: u32,
    /// The y of the last plain button drawn. Read by the buttons that borrow it.
    pub button_y: u32,
    /// What the screen being left returns: -1 on going back (`menu::go_back`), or the replay
    /// slot a naming screen picked (`menu::save_replay_loop`).
    pub leave_result: u32,
    /// What the item's action leaves: 2 = stay on the screen as it is (`menu::screen_loop`).
    pub action_mode: u32,
    /// The screen was left with Escape.
    pub escape_pressed: bool,
}

impl DrawWork {
    pub const fn new() -> Self {
        DrawWork(GameThread::new(RefCell::new(DrawValues {
            blink: true,
            cursor_on: true,
            button_x: 0x0,
            button_y: 0x0,
            leave_result: 0x0,
            action_mode: 0x0,
            escape_pressed: false,
        })))
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> DrawValues {
        *self.0.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, DrawValues> {
        self.0.get().borrow_mut()
    }
}
