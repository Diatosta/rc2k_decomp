//! The screens' stores in one struct, reached through `screens()` by the code that does not
//! take it as a parameter. (Not `menu::owner::Screens`, the menu's own group.)

use crate::screens::controls_store::ControlsScreen;
use crate::session::GameThread;

/// The screens' state.
pub struct ScreensOwner {
    /// The controls page's variables (history, learning).
    pub controls: ControlsScreen,
}

impl ScreensOwner {
    pub const fn new() -> Self {
        ScreensOwner {
            controls: ControlsScreen::new(),
        }
    }
}

/// The one instance, the game's thread's alone (the controls page runs in the menus).
/// Reached through `screens()`.
pub static SCREENS_OWNER: GameThread<ScreensOwner> = GameThread::new(ScreensOwner::new());

/// The screens' state, for a caller that does not take it as a parameter yet.
pub fn screens() -> &'static ScreensOwner {
    SCREENS_OWNER.get()
}
