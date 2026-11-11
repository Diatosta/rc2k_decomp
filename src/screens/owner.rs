//! Module screens' owner: the screens' stores in one struct, reached
//! through `screens()` by the code that does not take it as a parameter. (Not
//! `menu::owner::Screens`, the menu's own group.)

use crate::screens::controls_store::ControlsScreen;

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

/// The one instance. Reached through `screens()`; `moved` registers its stores' places.
pub static SCREENS_OWNER: ScreensOwner = ScreensOwner::new();

/// The screens' state, for a caller that does not take it as a parameter yet.
pub fn screens() -> &'static ScreensOwner {
    &SCREENS_OWNER
}
