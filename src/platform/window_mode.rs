//! How the window is shown: as a window, over the whole display with the display set to the
//! race's size (full screen), or over the whole desktop without a mode change (borderless; the
//! frame is shown scaled in it, its shape kept). Kept in `rc2k.toml` (`window_mode`), changed on
//! the graphics page and put into effect at once.

use std::sync::atomic::{AtomicU8, Ordering::Relaxed};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WindowMode {
    #[default]
    Window,
    FullScreen,
    Borderless,
}

impl WindowMode {
    pub const ALL: [WindowMode; 3] = [
        WindowMode::Window,
        WindowMode::FullScreen,
        WindowMode::Borderless,
    ];

    /// The name `rc2k.toml` keeps.
    pub fn name(self) -> &'static str {
        match self {
            WindowMode::Window => "window",
            WindowMode::FullScreen => "fullscreen",
            WindowMode::Borderless => "borderless",
        }
    }

    /// The mode a stored name means (a window for any other).
    pub fn of(name: &str) -> WindowMode {
        WindowMode::ALL
            .into_iter()
            .find(|m| m.name() == name.trim().to_ascii_lowercase())
            .unwrap_or_default()
    }

    /// The next mode, either way round.
    pub fn step(self, up: bool) -> WindowMode {
        let at = WindowMode::ALL.iter().position(|&m| m == self).unwrap_or(0);
        let n = WindowMode::ALL.len();
        WindowMode::ALL[if up { (at + 1) % n } else { (at + n - 1) % n }]
    }

    fn code(self) -> u8 {
        WindowMode::ALL.iter().position(|&m| m == self).unwrap_or(0) as u8
    }
}

static MODE: AtomicU8 = AtomicU8::new(0);

pub fn get() -> WindowMode {
    WindowMode::ALL[usize::from(MODE.load(Relaxed)).min(WindowMode::ALL.len() - 1)]
}

/// The mode kept (the window is changed by `platform::apply_window_mode`).
pub fn set(mode: WindowMode) {
    MODE.store(mode.code(), Relaxed);
}

/// Alt+Enter: a window goes to full screen, anything else back to a window.
pub fn toggled() -> WindowMode {
    if get() == WindowMode::Window {
        WindowMode::FullScreen
    } else {
        WindowMode::Window
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_steps() {
        assert_eq!(WindowMode::of("FullScreen"), WindowMode::FullScreen);
        assert_eq!(WindowMode::of("nonsense"), WindowMode::Window);
        assert_eq!(WindowMode::Window.step(false), WindowMode::Borderless);
        assert_eq!(WindowMode::Borderless.step(true), WindowMode::Window);
    }
}
