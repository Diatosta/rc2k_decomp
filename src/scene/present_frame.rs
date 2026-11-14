//! Callers: `race::loading_bar_frame` and the game's frame code.

/// The frame shown (`gfx::show_frame`, the platform's): false when it was not.
pub fn present_frame(o: crate::owners::Owners<'static>) -> bool {
    crate::platform::show_frame_now(o)
}
