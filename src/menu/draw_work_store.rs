//! What the screens' drawing and loops hand on between frames and screens: the blink phases, the
//! item action's outcome, the Escape flag, and three carries (the plain buttons' last place and
//! the leaving value). One static per `Screens`.

use crate::memory::session::{PlainDword, PlainFlag};

/// The drawing work variables.
#[derive(rc2k_derive::Cells)]
pub struct DrawWork {
    /// 1 in the visible half of a blink (`menu::update_blink`).
    pub blink: PlainFlag,
    /// 1 in the visible half of the text cursor's blink.
    pub cursor_on: PlainFlag,
    /// The x a plain button's drawing ends at. The buttons that borrow their place start from it
    /// (`screens::buttons`), and the best-times fill leaves its sum here (`saves::default_times`).
    pub button_x: PlainDword,
    /// The y of the last plain button drawn. Read by the buttons that borrow it.
    pub button_y: PlainDword,
    /// What the screen being left returns: -1 on going back (`menu::go_back`), or the replay
    /// slot a naming screen picked (`menu::save_replay_loop`).
    pub leave_result: PlainDword,
    /// What the item's action leaves: 2 = stay on the screen as it is (`menu::screen_loop`).
    pub action_mode: PlainDword,
    /// The screen was left with Escape.
    pub escape_pressed: PlainFlag,
}

impl DrawWork {
    pub const fn new() -> Self {
        DrawWork {
            blink: PlainFlag::new(true),
            cursor_on: PlainFlag::new(true),
            button_x: PlainDword::new(0x0),
            button_y: PlainDword::new(0x0),
            leave_result: PlainDword::new(0x0),
            action_mode: PlainDword::new(0x0),
            escape_pressed: PlainFlag::new(false),
        }
    }
}
