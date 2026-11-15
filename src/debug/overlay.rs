//! The debug pages' keys: with Ctrl held, F11 shows or hides the debug UI (`debug::ui`), F12
//! freezes the race or lets it go (`drive::race_step` skips its physics ticks). `input::end_frame`
//! clears the freeze every frame, so it is kept here and put back there (`words`); the page
//! stays 0 (the developers' pages are the UI's tabs now).

use core::sync::atomic::{AtomicU32, Ordering};

/// The keys (scan codes in the key tables): Ctrl, F11, F12.
const CONTROL_KEY: usize = 0x1D;
const UI_KEY: usize = 0x57;
const FREEZE_KEY: usize = 0x58;

/// 1 while the race is frozen.
static FROZEN: AtomicU32 = AtomicU32::new(0);

/// Whether the race is frozen.
pub fn frozen() -> bool {
    FROZEN.load(Ordering::Relaxed) != 0
}

/// The race frozen or let go.
pub fn set_frozen(on: bool) {
    FROZEN.store(on as u32, Ordering::Relaxed);
    crate::logfile::program_log::log(&format!("debug: race frozen {}", on as u32));
}

/// Ctrl+F11 (`platform`, from the window's events).
pub fn toggle_ui() {
    let open = crate::debug::imgui_sdl::toggle();
    crate::logfile::program_log::log(&format!(
        "debug: the UI {}",
        if open { "shown" } else { "hidden" }
    ));
}

/// The keys, once a frame (`input::update`, before the game reads the frame's keys): with
/// Ctrl down, F11 pressed the UI shown or hidden, F12 pressed the race frozen or going; and
/// both taken out of the key tables, so that the game does not see them (F12 is its map).
pub fn take_keys(devices: &crate::input::store::Devices) {
    if devices.keys_now.get(CONTROL_KEY) == 0 {
        return;
    }
    let pressed = |key| devices.keys_pressed.get(key) & 1 != 0;
    let freeze = pressed(FREEZE_KEY);
    for key in [UI_KEY, FREEZE_KEY] {
        devices.keys_down.set(key, 0);
        devices.keys_now.set(key, 0);
        devices.keys_pressed.set(key, 0);
    }
    if freeze {
        set_frozen(!frozen());
    }
}

/// What `input::end_frame` leaves in the debug page and freeze words: 0 and the freeze.
pub fn words() -> [u32; 2] {
    [0, FROZEN.load(Ordering::Relaxed)]
}
