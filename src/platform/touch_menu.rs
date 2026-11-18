//! The touch screen in the menus (phones, tablets, any touch screen): SDL's finger events into
//! the gestures of `rc2k_core::input::menu_touch`, which become the keys the menus already take
//! (a drag is the arrow keys, Back is Escape), and a tap on the menu on show, which highlights
//! the item under the finger and chooses it as Enter does. Screens that run their own loops
//! (lists, the car carousel, text fields) keep their own keys: a tap there is Enter. A tap past
//! the ends of a list goes back. The Back chip is drawn over the frame once a finger has touched
//! the screen.

use crate::input::menu_pad::MenuAction;
use crate::owners::Owners;
use crate::platform::ffi::sdl;
use rc2k_core::input::menu_touch::{
    Gesture, Gestures, Phase, Row, Screen, Tapped, back_chip, tapped,
};
use rc2k_core::menu::navigation::CAN_BE_CHOSEN;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// A link to no item.
const NO_LINK: u32 = 0x6F;

/// How far from an item's row a tap still chooses it, in the menu frame's pixels (the rows are
/// about 29 apart).
const REACH: f32 = 16.0;

static GESTURES: Mutex<Gestures> = Mutex::new(Gestures::new());

/// Whether one of the installer's pages is running: its items are not the menus' (it keeps its
/// own pieces and highlight), so a tap is left for the page to take (`take_installer_tap`).
static INSTALLER: AtomicBool = AtomicBool::new(false);

/// The last tap on the installer's page not yet taken, in the frame's pixels.
static INSTALLER_TAP: Mutex<Option<(f32, f32)>> = Mutex::new(None);

/// A finger has touched the screen since the program started: the overlay is worth showing.
static TOUCHED: AtomicBool = AtomicBool::new(false);

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
    TOUCHED.store(true, Relaxed);
    let Some(screen) = screen_of(window) else {
        return;
    };
    let gestures = GESTURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .finger(&screen, id, phase, x * screen.width, y * screen.height, now);
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

/// A tap at (`x`, `y`) in the window's pixels.
fn tap(o: &Owners<'static>, window: sdl::Window, x: f32, y: f32) {
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
    let Some((fx, fy)) = point else {
        // Beside the picture: nothing.
        return;
    };
    if INSTALLER.load(Relaxed) {
        *INSTALLER_TAP
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((fx, fy));
        return;
    }
    let menus = o.menus;
    let rows = menus
        .screens
        .loop_flags
        .menu
        .get()
        .ok()
        .map(|menu| rows_of(o, menu))
        .unwrap_or_default();
    match tapped(&rows, fy, REACH) {
        Tapped::Item(number) => {
            menus.screens.loop_flags.item.set(number);
            press(o, MenuAction::Enter);
        }
        Tapped::Outside => press(o, MenuAction::Back),
        Tapped::Enter => press(o, MenuAction::Enter),
    }
}

/// The rows of the menu on show that can be chosen. None when the menu's items link only
/// sideways (the main menu's carousel has one item on show): a tap chooses it.
fn rows_of(o: &Owners<'static>, menu: crate::menu::menus_store::MenuId) -> Vec<Row> {
    let items = menu.items(&o.menus.tables.menus);
    if items
        .iter()
        .all(|item| item.fixed().links[..2].iter().all(|&l| l == NO_LINK))
    {
        return Vec::new();
    }
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.skip_mark.get() == CAN_BE_CHOSEN)
        .map(|(n, item)| Row {
            number: n as u32 + 1,
            // The text's top, and half the text's height.
            y: item.text_y() as i32 as f32 + 8.0,
        })
        .collect()
}

/// The Back chip over the frame, in the menus, once the screen has been touched.
pub fn draw_back_chip(renderer: sdl::Renderer, window: sdl::Window) {
    if !TOUCHED.load(Relaxed) {
        return;
    }
    let Some(screen) = screen_of(window) else {
        return;
    };
    let r = back_chip(&screen);
    renderer.fill_blended([255, 255, 255, 50], [r.x, r.y, r.w, r.h]);
    let scale = r.h * 0.34 / 8.0;
    renderer.text_blended(
        [255, 255, 255, 190],
        r.x + r.w * 0.1,
        r.y + r.h * 0.33,
        scale,
        "BACK",
    );
}

/// While it lives, the installer's page is the one taps are for.
pub struct InstallerPage;

impl InstallerPage {
    pub fn begin() -> Self {
        INSTALLER.store(true, Relaxed);
        InstallerPage
    }
}

impl Drop for InstallerPage {
    fn drop(&mut self) {
        INSTALLER.store(false, Relaxed);
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
