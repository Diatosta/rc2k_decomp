//! The window and renderer on the handheld, whose frames go to the framebuffer through its own
//! GL ES backend: SDL's dummy video driver, a plain renderer for the window, and none of the
//! desktop's window modes or second backend. `window_system.rs` has the same names for the other
//! systems.

use super::*;

/// The window that has the keyboard: the game's own.
pub(crate) fn input_window(window: sdl::Window) -> sdl::Window {
    window
}

/// The window mode is not changed: there is one window, the panel's.
pub fn apply_window_mode() {}

/// The window's renderer: SDL's own; the GPU starts with the first frame (`render::show_on_panel`).
pub(super) fn open_renderer(window: sdl::Window, _size: (usize, usize)) -> Option<sdl::Renderer> {
    sdl::create_renderer(window)
}

/// The handheld's frames go to the framebuffer (`handheld`).
pub(super) fn before_video() {
    sdl::use_dummy_video_driver();
}

/// Nothing to place or put into a mode.
pub(super) fn settle_window(
    _window: sdl::Window,
    _window_size: (usize, usize),
    _window_before: Option<(usize, usize)>,
    _window_changed: bool,
) {
}

/// There is no second backend with a window of its own.
pub(super) fn shows_itself() -> bool {
    false
}

/// The GPU backend's targets are the panel's: nothing to size here.
pub(super) fn size_gpu_targets(_width: u32, _height: u32) {}

/// Nothing to say of the 3D's place: the panel's backend is the GPU.
pub(super) fn renderer_note() -> &'static str {
    ""
}

/// The menus' mode: the one the options keep.
pub(super) fn menu_mode(race_mode: u32, menu_mode: u32) -> u32 {
    let _ = race_mode;
    menu_mode
}

/// Whether the menus' frame has the race's window size: not on the handheld, whose panel is the
/// frame's own size.
pub(super) const MENU_WINDOW_IS_RACE_SIZED: bool = false;
