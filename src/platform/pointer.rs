//! How the player points: with a mouse (and a keyboard) or with a finger on a touch screen.
//!
//! Pages ask here for the sizes and texts that differ (taller buttons with more room between
//! them, a hint that names the slides instead of the keys) instead of testing the build's
//! target themselves. The answer is fixed for a build: the phones and tablets are touch builds,
//! and `RC2K_TOUCH_BUILD=1` makes one for any other system. The code that handles the taps
//! themselves (`shell::touch_menu`) exists only in a touch build, so it stays behind its own
//! condition.

/// How the player points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// A mouse or a pad, with the keyboard's keys.
    Pointer,
    /// A finger on the screen.
    Touch,
}

/// The pointing style of this build.
pub const fn style() -> Style {
    if cfg!(touch_screen) {
        Style::Touch
    } else {
        Style::Pointer
    }
}

/// Whether the player points with a finger.
pub const fn is_touch() -> bool {
    matches!(style(), Style::Touch)
}

/// `touch` on a touch screen, `pointer` otherwise (for the sizes of a page's parts).
pub const fn by_style(pointer: i32, touch: i32) -> i32 {
    if is_touch() { touch } else { pointer }
}
