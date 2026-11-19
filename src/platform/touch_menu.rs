//! The touch screen in the menus (phones, tablets, any touch screen): SDL's finger events into
//! the gestures of `rc2k_core::input::menu_touch`, which become the keys the menus already take:
//! a drag is the arrow keys, a single tap is Enter on what is highlighted (wherever the finger
//! was; it is sent once the double-tap window has passed, so a double tap never also chooses),
//! a double tap is Escape. Nothing else acts. The pages that take their own taps
//! (`InstallerPage`) get a tap when the finger lifts; one they act on (`tap_used`) is not the
//! first of a double tap.

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

/// When the Escape key held for a double tap during a replay is let go (0: none held).
static ESCAPE_UNTIL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// How long a double tap holds Escape down during a replay: the race's loop reads the key's
/// state at its own pace, as it does a key on a keyboard.
const ESCAPE_HOLD_MS: u64 = 200;

/// Keys from gestures waiting for the menus to take the one before: the menus hold one key at a
/// time, and one quick swipe can give several steps at once.
/// Each with the menu it was given on: a key is not carried to another screen.
static QUEUED: Mutex<
    std::collections::VecDeque<(MenuAction, Option<crate::menu::menus_store::MenuId>)>,
> = Mutex::new(std::collections::VecDeque::new());

/// The most keys kept waiting (a long swipe's steps; more are dropped).
const QUEUE_ROOM: usize = 8;

/// The game's key code of Escape.
const ESCAPE_KEY: u8 = 0x01;

/// When a dropped or ignored touch was last logged (ms; 0: never) and how many were left out
/// since: a phone's log shows why the menus stopped answering without one line a touch.
static NOTED_AT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static NOTED: AtomicUsize = AtomicUsize::new(0);

/// Notes that a touch was dropped or ignored for `reason`, at most one log line every 2 s
/// (with how many came before it).
fn note(reason: &str, now: u64) {
    let count = NOTED.fetch_add(1, Relaxed) + 1;
    let last = NOTED_AT.load(Relaxed);
    if last != 0 && now.saturating_sub(last) < 2000 {
        return;
    }
    NOTED_AT.store(now.max(1), Relaxed);
    NOTED.store(0, Relaxed);
    crate::logfile::program_log::log(&format!(
        "touch menu: {reason} ({count} such since the last line)"
    ));
}

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
        if phase == Phase::Up {
            forget(id);
        }
        note("the window's size is unknown, a finger is ignored", now);
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
        forget_queued();
        note(
            "the screen changed under a finger, the touches are dropped",
            now,
        );
    }
    if !recogniser.active() && !recogniser.waiting() {
        *on_screen = menu;
        if phase != Phase::Down {
            return;
        }
    }
    let was_active = recogniser.active();
    let gestures = recogniser.finger(&screen, id, phase, x * screen.width, y * screen.height, now);
    let stale = recogniser.take_stale();
    if stale > 0 {
        note("a finger's lifting never came, it is dropped", now);
    } else if was_active && phase == Phase::Down {
        note("a second finger went down, two fingers do nothing", now);
    }
    drop((recogniser, on_screen));
    act(o, window, gestures, now);
}

/// Called every turn of the event loop: the single tap whose double-tap window is over is chosen.
pub fn tick(o: &Owners<'static>, window: sdl::Window, now: u64) {
    if WATCH_LOOKS.load(Relaxed) > 0 {
        trace(&format!(
            "events read, key {:#x} waiting, menu {:?}",
            crate::input::take_key::pending_key(o.input).load(std::sync::atomic::Ordering::SeqCst),
            o.menus.screens.loop_flags.menu.get().ok()
        ));
    }
    hand_over_queued(o);
    let until = ESCAPE_UNTIL.load(Relaxed);
    if until != 0 && now >= until {
        ESCAPE_UNTIL.store(0, Relaxed);
        o.input.devices.set_key_state(ESCAPE_KEY, 0);
    }
    let menu = o.menus.screens.loop_flags.menu.get().ok();
    let mut recogniser = GESTURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // A finger whose lifting was never seen would make the next one a pair.
    if recogniser.drop_stale(now) > 0 {
        recogniser.take_stale();
        note("a finger's lifting never came, it is dropped", now);
    }
    if !recogniser.waiting() {
        return;
    }
    // The screen changed while the tap waited: it was meant for the one it went down on.
    let on_screen = *DOWN_ON
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if on_screen != menu {
        recogniser.release_all();
        forget_queued();
        note("the screen changed under a waiting tap, it is dropped", now);
        return;
    }
    let gestures = recogniser.tick(now);
    drop(recogniser);
    act(o, window, gestures, now);
}

/// One diagnostic line about the menus' touch (while touches go missing on a phone).
pub fn trace(text: &str) {
    crate::logfile::program_log::log(&format!(
        "touch trace {}: {text}",
        crate::platform::os::ticks()
    ));
}

/// Diagnostic: the menus' looks for a key are each logged while this many remain (set when a
/// touch key has to wait).
pub static WATCH_LOOKS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn act(o: &Owners<'static>, window: sdl::Window, gestures: Vec<Gesture>, now: u64) {
    for gesture in &gestures {
        trace(&format!(
            "{gesture:?} on menu {:?}, item {:?}, page count {}",
            o.menus.screens.loop_flags.menu.get().ok(),
            o.menus.screens.loop_flags.item.get(),
            INSTALLER.load(Relaxed)
        ));
    }
    for gesture in gestures {
        match gesture {
            Gesture::Up => press(o, MenuAction::Up),
            Gesture::Down => press(o, MenuAction::Down),
            Gesture::Left => press(o, MenuAction::Left),
            Gesture::Right => press(o, MenuAction::Right),
            Gesture::Back => {
                press(o, MenuAction::Back);
                // A replay being shown reads Escape's state, not the key taken: the double tap
                // also holds the key down a moment, as a keyboard's Escape does.
                if o.race.course.numbers.get().in_race.is_set() {
                    o.input.devices.set_key_state(ESCAPE_KEY, 0xFF);
                    ESCAPE_UNTIL.store(now + ESCAPE_HOLD_MS, Relaxed);
                }
            }
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

/// A finger's lifting that could not reach `finger` (no window): the finger leaves the list.
pub fn forget(id: u64) {
    GESTURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .forget(id);
}

/// Every finger lifted (a race began, the app went to the background).
pub fn release_all() {
    GESTURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .release_all();
    forget_queued();
}

/// The keys still queued dropped: they were meant for a screen that has gone.
fn forget_queued() {
    QUEUED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
}

fn press(o: &Owners<'static>, action: MenuAction) {
    let waiting =
        crate::input::take_key::pending_key(o.input).load(std::sync::atomic::Ordering::SeqCst);
    let mut queued = QUEUED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if waiting != 0 || !queued.is_empty() {
        let looked = u64::from(crate::platform::os::ticks())
            .saturating_sub(crate::menu::handle_keys::LAST_LOOK.load(Relaxed));
        trace(&format!(
            "{action:?} queued behind key {waiting:#x} ({} queued); the menus looked {looked} ms ago, keys off {}",
            queued.len(),
            crate::menu::handle_keys::LAST_LOOK_OFF.load(Relaxed)
        ));
        WATCH_LOOKS.store(40, Relaxed);
        if queued.len() < QUEUE_ROOM {
            queued.push_back((action, o.menus.screens.loop_flags.menu.get().ok()));
        }
        return;
    }
    drop(queued);
    trace(&format!(
        "{action:?} given at once ({:?})",
        std::thread::current().id()
    ));
    super::events::menu_action(o.input, action);
}

/// The next queued key handed over once the menus took the one before.
fn hand_over_queued(o: &Owners<'static>) {
    if crate::input::take_key::pending_key(o.input).load(std::sync::atomic::Ordering::SeqCst) != 0 {
        return;
    }
    let next = QUEUED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .pop_front();
    if let Some((action, menu)) = next {
        if menu == o.menus.screens.loop_flags.menu.get().ok() {
            trace(&format!("queued {action:?} handed over on menu {menu:?}"));
            super::events::menu_action(o.input, action);
        } else {
            trace(&format!(
                "queued {action:?} for menu {menu:?} forgotten: now on {:?}",
                o.menus.screens.loop_flags.menu.get().ok()
            ));
            forget_queued();
        }
    }
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
        set_immediate(true);
        *INSTALLER_TAP
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        InstallerPage
    }
}

impl Drop for InstallerPage {
    fn drop(&mut self) {
        if INSTALLER.fetch_sub(1, Relaxed) == 1 {
            set_immediate(false);
        }
        *INSTALLER_TAP
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }
}

/// Pages that take their own taps get them when the finger lifts, not after the double-tap
/// window; the game's menus keep the window.
fn set_immediate(on: bool) {
    GESTURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_immediate(on);
}

/// The page acted on the tap it took (it was on a button): the tap is not the first of a double
/// tap (a double tap off a button is Back).
pub fn tap_used() {
    GESTURES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .tap_used();
}

/// The tap on the installer's page since the last call, if any: where, in the frame's pixels.
pub fn take_installer_tap() -> Option<(f32, f32)> {
    INSTALLER_TAP
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}
