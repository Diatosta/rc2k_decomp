//! The touch screen in the menus (phones, tablets, any touch screen): SDL's finger events into
//! the gestures of `rc2k_core::input::menu_touch`, which become the keys the menus already take:
//! a drag is the arrow keys, a single tap is Enter on what is highlighted (wherever the finger
//! was; it is sent once the double-tap window has passed, so a double tap never also chooses),
//! a double tap is Escape. Nothing else acts.

use crate::input::menu_pad::MenuAction;
use crate::owners::Owners;
use crate::platform::ffi::sdl;
use rc2k_core::input::menu_touch::{Gesture, Gestures, Phase, Screen};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

static GESTURES: Mutex<Gestures> = Mutex::new(Gestures::new());

/// How many pages that take their own taps are running (the installer's pages, the port's own
/// dialog-style pages: they keep their own pieces and highlight, not the menus' items), so a tap
/// is left for the innermost page to take (`take_installer_tap`). A page opened by another
/// (the save code page's scan, a notice over it) counts one more; each leaving counts one less.
static INSTALLER: AtomicUsize = AtomicUsize::new(0);

/// The last tap on the installer's page not yet taken, in the frame's pixels.
static INSTALLER_TAP: Mutex<Option<(f32, f32)>> = Mutex::new(None);

/// The menu on show when the fingers in progress (or the tap waiting for its window) went down.
static DOWN_ON: Mutex<Option<crate::menu::menus_store::MenuId>> = Mutex::new(None);

/// The window's size and safe area as the screen the gestures are sized by.
pub fn screen_of(window: sdl::Window) -> Option<Screen> {
    let (width, height) = window.pixel_size()?;
    let (width, height) = (width as f32, height as f32);
    let mut screen = Screen::new(width, height);
    if let Some([x, y, w, h]) = window.safe_area() {
        screen.left = x.max(0.0);
        screen.top = y.max(0.0);
        screen.right = (width - x - w).max(0.0);
        screen.bottom = (height - y - h).max(0.0);
    }
    Some(screen)
}

/// A finger's change (`x`, `y`: parts of the window) in the menus: what it means is done.
pub fn finger(
    o: &Owners<'static>,
    window: sdl::Window,
    id: u64,
    phase: Phase,
    x: f32,
    y: f32,
    now: u64,
) {
    let Some(screen) = screen_of(window) else {
        return;
    };
    let menu = o.menus.screens.loop_flags.menu.get().ok();
    let mut recogniser = GESTURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // A tap acts once, when the finger lifts, and only on the screen it went down on: a screen
    // change drops the touches in progress.
    let mut on_screen = DOWN_ON
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if (recogniser.active() || recogniser.waiting()) && *on_screen != menu {
        recogniser.release_all();
    }
    if !recogniser.active() && !recogniser.waiting() {
        *on_screen = menu;
        if phase != Phase::Down {
            return;
        }
    }
    let gestures = recogniser.finger(&screen, id, phase, x * screen.width, y * screen.height, now);
    drop((recogniser, on_screen));
    act(o, window, gestures);
}

/// Called every turn of the event loop: the single tap whose double-tap window is over is chosen.
pub fn tick(o: &Owners<'static>, window: sdl::Window, now: u64) {
    let menu = o.menus.screens.loop_flags.menu.get().ok();
    let mut recogniser = GESTURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !recogniser.waiting() {
        return;
    }
    // The screen changed while the tap waited: it was meant for the one it went down on.
    let on_screen = *DOWN_ON
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if on_screen != menu {
        recogniser.release_all();
        return;
    }
    let gestures = recogniser.tick(now);
    drop(recogniser);
    act(o, window, gestures);
}

fn act(o: &Owners<'static>, window: sdl::Window, gestures: Vec<Gesture>) {
    for gesture in gestures {
        match gesture {
            Gesture::Up => press(o, MenuAction::Up),
            Gesture::Down => press(o, MenuAction::Down),
            Gesture::Left => press(o, MenuAction::Left),
            Gesture::Right => press(o, MenuAction::Right),
            Gesture::Back => press(o, MenuAction::Back),
            Gesture::Tap { x, y } => tap(o, window, x, y),
        }
    }
}

/// Whether sideways drags give the key on the finger's side (a page that is not a carousel).
pub fn set_direct_sideways(on: bool) {
    GESTURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_direct_sideways(on);
}

/// Every finger lifted (a race began, the app went to the background).
pub fn release_all() {
    GESTURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .release_all();
}

fn press(o: &Owners<'static>, action: MenuAction) {
    super::events::menu_action(o.input, action);
}

/// A single tap at (`x`, `y`) in the window's pixels: Enter, wherever it was. The installer's
/// page, which keeps pieces of its own, is given where it was in the frame's pixels instead.
fn tap(o: &Owners<'static>, window: sdl::Window, x: f32, y: f32) {
    if INSTALLER.load(Relaxed) == 0 {
        press(o, MenuAction::Enter);
        return;
    }
    // The one mapping from the window to the frame (`platform::safe_area`).
    let frame = crate::platform::SCREEN.lock().ok().and_then(|screen| {
        screen
            .as_ref()
            .map(|s| ((s.width as u32, s.height as u32), s.menu))
    });
    let point = frame
        .zip(window.pixel_size())
        .and_then(|((frame, menu), size)| {
            crate::platform::safe_area::window_to_frame(
                (f64::from(x), f64::from(y)),
                frame,
                size,
                menu,
            )
            .map(|(fx, fy)| (fx as f32, fy as f32))
        });
    if let Some(point) = point {
        *INSTALLER_TAP
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(point);
    }
}

/// While it lives, the page (the installer's, or a dialog-style page of the port's own) is the
/// one taps are for. Pages may nest.
pub struct InstallerPage;

impl InstallerPage {
    pub fn begin() -> Self {
        INSTALLER.fetch_add(1, Relaxed);
        *INSTALLER_TAP
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        InstallerPage
    }
}

impl Drop for InstallerPage {
    fn drop(&mut self) {
        INSTALLER.fetch_sub(1, Relaxed);
        *INSTALLER_TAP
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }
}

/// The tap on the installer's page since the last call, if any: where, in the frame's pixels.
pub fn take_installer_tap() -> Option<(f32, f32)> {
    INSTALLER_TAP
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}
