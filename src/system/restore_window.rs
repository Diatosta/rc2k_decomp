//! Restores the game's window if it is minimised (used by `audio::set_mode` and the network's
//! calls). The window is SDL's (`platform::restore_window`, done on the window's thread at its
//! next pump).

pub fn restore_window() {
    crate::platform::events::restore_window();
}
