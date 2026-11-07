//! Callers: `race::loading_bar_frame` and the game's (0x4395FB, 0x439605, 0x4426A0, 0x4435DD).
//! The registers above are the original's; a call by address goes through `routes.rs`, which
//! maps them.

/// The frame shown (`gfx::show_frame`, the platform's): false when it was not (the
/// original's carry set). Every register kept.
pub fn present_frame() -> bool {
    crate::platform::show_frame_now()
}
