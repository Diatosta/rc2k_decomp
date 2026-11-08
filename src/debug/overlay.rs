//! The debug pages' keys (owner's choice of a key, 2026-10-30): with Ctrl held, F11 shows or
//! hides the debug UI (`debug::ui`), F12 freezes the race or lets it go
//! (`g_debugFreezeRace`: `drive::race_step` skips its physics ticks). The game's code that
//! set `g_debugPage` and `g_debugFreezeRace` is not in RAL.EXE (`input::end_frame` sets
//! both 0 every frame): the freeze is kept here and put back by `input::end_frame`
//! (`words`); the page stays 0 (the original's pages are the UI's tabs now).

use crate::input::owner::input;
use core::sync::atomic::{AtomicU32, Ordering};

/// The keys (scan codes in `g_keysNow` / `g_keysPressed`): Ctrl, F11, F12.
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
    crate::framework::log(&format!("debug: race frozen {}", on as u32));
}

/// Ctrl+F11 (`platform`, from the window's events).
pub fn toggle_ui() {
    let open = crate::debug::imgui_sdl::toggle();
    crate::framework::log(&format!(
        "debug: the UI {}",
        if open { "shown" } else { "hidden" }
    ));
}

/// The keys, once a frame (`input::update`, before the game reads the frame's keys): with
/// Ctrl down, F11 pressed the UI shown or hidden, F12 pressed the race frozen or going; and
/// both taken out of `g_keysDown` (the bindings read it), `g_keysNow` and `g_keysPressed`,
/// so that the game does not see them (F12 is its map).
pub fn take_keys() {
    let devices = &input().devices;
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

/// What `input::end_frame` leaves in `g_debugPage` and `g_debugFreezeRace`: 0 and the
/// freeze (both 0, the original's, until it is used).
pub fn words() -> [u32; 2] {
    [0, FROZEN.load(Ordering::Relaxed)]
}
