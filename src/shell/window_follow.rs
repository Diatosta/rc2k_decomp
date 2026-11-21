//! The player's resizing of the window: in window mode the window can be dragged to any size,
//! and the race is then drawn at that size. A resize is taken up once the window has stayed
//! still for `SETTLE_MS` (the end of a drag). The new size becomes the race's size; in a running
//! race the frame is made again at once at that size (the stage's textures and the race's state
//! stay), so the picture fills the window as the menus' does. Between a race's loading and its
//! run the resize waits. Full screen and borderless are never followed: their size is the
//! display's.

use crate::platform::window_mode::{self, WindowMode};
use crate::program_log::log;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// How long the window must have kept its size before it is taken up.
const SETTLE_MS: u64 = 400;

/// No resize waits.
const NONE: u64 = u64::MAX;

/// When (in `os::ticks` ms) the last resize event came, or `NONE`.
static LAST_EVENT: AtomicU64 = AtomicU64::new(NONE);

/// SDL says the window changed size (its own doing or ours: told apart by `size_to_take`).
pub(super) fn note_resize(now: u64) {
    LAST_EVENT.store(now, Relaxed);
}

/// The size to take up: the window's `actual` size when it differs from the `shown` size and is
/// one the options can keep.
fn size_to_take(actual: (u32, u32), shown: (u32, u32)) -> Option<(u32, u32)> {
    let (w, h) = actual;
    (actual != shown && (320..=16384).contains(&w) && (240..=16384).contains(&h)).then_some(actual)
}

/// Whether the settling time has passed since an event at `last`.
fn settled(last: u64, now: u64) -> bool {
    last != NONE && now.saturating_sub(last) >= SETTLE_MS
}

/// Takes up the window's size, if a resize has settled and a menu or a running race is shown: the
/// size becomes the race's size (kept in the options), the graphics page's list is made again
/// with it, and a running race's frame is made again at it.
pub fn follow(o: crate::owners::Owners<'static>, now: u64) {
    if !crate::platform::ffi::sdl::RESIZABLE || !settled(LAST_EVENT.load(Relaxed), now) {
        return;
    }
    let racing = o.race.course.in_race();
    let (size, frame_is_race) = {
        let Ok(mut screen) = crate::platform::SCREEN.lock() else {
            return;
        };
        let Some(screen) = screen.as_mut() else {
            return;
        };
        let shown = crate::platform::input_window(screen.window);
        if window_mode::get() != WindowMode::Window || shown.is_full_screen() {
            LAST_EVENT.store(NONE, Relaxed);
            return;
        }
        // Between a race's loading and its end the resize waits (its frame is made again once
        // the race runs).
        if !screen.menu && !racing {
            return;
        }
        LAST_EVENT.store(NONE, Relaxed);
        let Some(size) = shown.window_size().and_then(|actual| {
            size_to_take(
                actual,
                (screen.window_size.0 as u32, screen.window_size.1 as u32),
            )
        }) else {
            return;
        };
        screen.window_size = (size.0 as usize, size.1 as usize);
        (size, !screen.menu)
    };
    // The screen's lock is let go: making the list asks the display's modes through it.
    let (gfx, system) = (o.gfx, o.system);
    let display = &gfx.display;
    let race = display.mode.race_mode.get();
    let Some(mode) = crate::gfx::monitor_sizes::race_mode_for(&display.modes, race, size.0, size.1)
    else {
        log("platform: the window's new size cannot be kept (no number left for it)");
        return;
    };
    crate::gfx::monitor_sizes::set_window(Some(size));
    display.mode.race_mode.set(mode);
    display
        .mode
        .menu_mode
        .set(rc2k_core::gfx::screen_modes::menu_mode_for(mode));
    if let Err(why) =
        crate::gfx::screen_mode_choices::screen_mode_choices(gfx, &gfx.d3d, system, false)
    {
        log(&format!(
            "platform: the screen mode choices are not made again: {why:?}"
        ));
    }
    log(&format!(
        "platform: the window was resized to {} x {}: the race's size from now on",
        size.0, size.1
    ));
    if frame_is_race {
        // The 3D drawing noted so far, before the screen is changed.
        crate::d3d::device::flush();
        crate::platform::remake_race_frame_now(gfx, system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_resize_is_taken_once_still() {
        assert!(!settled(NONE, 10_000));
        assert!(!settled(1_000, 1_399));
        assert!(settled(1_000, 1_400));
    }

    #[test]
    fn only_a_new_size_the_options_can_keep_is_taken() {
        assert_eq!(size_to_take((1234, 777), (800, 600)), Some((1234, 777)));
        assert_eq!(size_to_take((800, 600), (800, 600)), None);
        assert_eq!(size_to_take((200, 600), (800, 600)), None);
        assert_eq!(size_to_take((800, 20000), (800, 600)), None);
    }
}
