//! SDL's events under the game: closing the window sets the quit switch, and the keys and
//! buttons the menus know become the game's pending key.

use crate::framework::log;
use crate::input::store::DEVICES;
use crate::session::FLAGS;
use sdl3_sys::everything::*;

/// An SDL event the program looks at.
enum Event {
    Quit,
    Focus(bool),
    Key {
        scancode: i32,
        down: bool,
    },
    Pad {
        button: SDL_GamepadButton,
        down: bool,
    },
    Other,
}

/// SDL's next event, if there is one.
fn next_event() -> Option<Event> {
    // SAFETY: an all-zero `SDL_Event` is valid (a union of plain records); SDL fills it, and
    // its `type` says which member the reads below take.
    unsafe {
        let mut event: SDL_Event = std::mem::zeroed();
        if !SDL_PollEvent(&raw mut event) {
            return None;
        }
        // Ctrl+F11, in the race and in the menus (which never run `input::update`), and
        // whatever has the UI's keyboard.
        if event.r#type == SDL_EVENT_KEY_DOWN.0
            && event.key.scancode == SDL_SCANCODE_F11
            && !event.key.repeat
            && event.key.r#mod.0 & SDL_KMOD_CTRL.0 != 0
        {
            crate::debug::overlay::toggle_ui();
            return Some(Event::Other);
        }
        // The debug UI's first (`debug::imgui_sdl`): what it takes the game does not see.
        if crate::debug::imgui_sdl::event(&event) {
            return Some(Event::Other);
        }
        let kind = event.r#type;
        Some(if kind == SDL_EVENT_QUIT.0 {
            Event::Quit
        } else if kind == SDL_EVENT_WINDOW_FOCUS_GAINED.0 {
            Event::Focus(true)
        } else if kind == SDL_EVENT_WINDOW_FOCUS_LOST.0 {
            Event::Focus(false)
        } else if kind == SDL_EVENT_JOYSTICK_ADDED.0 {
            crate::platform_input::pad_added(event.jdevice.which);
            Event::Other
        } else if kind == SDL_EVENT_GAMEPAD_BUTTON_DOWN.0 || kind == SDL_EVENT_GAMEPAD_BUTTON_UP.0 {
            Event::Pad {
                button: SDL_GamepadButton(event.gbutton.button as i32),
                down: event.gbutton.down,
            }
        } else if kind == SDL_EVENT_KEY_DOWN.0 || kind == SDL_EVENT_KEY_UP.0 {
            Event::Key {
                scancode: event.key.scancode.0,
                down: event.key.down,
            }
        } else {
            Event::Other
        })
    }
}

/// Set by `restore_window` (any thread): the window is restored at the next pump, on the
/// thread that made it.
static RESTORE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The game's window restored if it is minimised (`IsIconic` / `ShowWindow(SW_RESTORE)`),
/// at the next pump of SDL's events.
pub fn restore_window() {
    RESTORE.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// The window restored if asked and minimised (on the window's thread).
fn restore_if_asked() {
    if !RESTORE.swap(false, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let Ok(screen) = crate::platform::SCREEN.lock() else {
        return;
    };
    if let Some(screen) = screen.as_ref() {
        let window = std::ptr::with_exposed_provenance_mut::<SDL_Window>(screen.window);
        // SAFETY: the window SDL made, alive while `SCREEN` holds it (locked here).
        unsafe {
            if SDL_GetWindowFlags(window).0 & SDL_WINDOW_MINIMIZED.0 != 0 {
                SDL_RestoreWindow(window);
            }
        }
    }
}

/// How long the game has to leave after its window is closed.
const LEAVING_GRACE: std::time::Duration = std::time::Duration::from_secs(5);

/// Set while the installer runs: its copy goes on to its end after a close (a half-copied
/// install would be worse), so the watch below stays off.
static NO_CLOSE_WATCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The installer's: no ending `LEAVING_GRACE` after a close.
pub fn keep_running_after_close() {
    NO_CLOSE_WATCH.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Set while the installer copies: a close then waits for the copy's end (`QUIT_ASKED`),
/// so its progress is still drawn and shown.
static HOLD_QUIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static QUIT_ASKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The installer's copy begins (`true`) or ends: a close during it sets the quit switch at
/// its end.
pub fn hold_quit(on: bool) {
    use std::sync::atomic::Ordering::Relaxed;
    HOLD_QUIT.store(on, Relaxed);
    if !on && QUIT_ASKED.swap(false, Relaxed) {
        log("platform: the copy has ended: the quit switch is set");
        FLAGS.quitting.set(1);
        end_if_not_left();
    }
}

/// After the window's closing: the menus, races and films look at the quit switch and leave
/// (the original's window procedure only shut the screens down and posted `WM_QUIT`, which
/// `system::pump_messages` swallowed: nothing stopped, IDA 2026-10-28). A safety net for a
/// loop that still does not: if the program is running `LEAVING_GRACE` after the first
/// close, it ends, saving nothing: the game's thread may be in the tables a save reads.
fn end_if_not_left() {
    static WATCHING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if NO_CLOSE_WATCH.load(std::sync::atomic::Ordering::Relaxed)
        || WATCHING.swap(true, std::sync::atomic::Ordering::Relaxed)
    {
        return;
    }
    let started = std::thread::Builder::new()
        .name("window closed".into())
        .spawn(|| {
            std::thread::sleep(LEAVING_GRACE);
            crate::standalone::finish(
                "the window was closed and the game had not left 5 s later",
                0,
            )
        });
    if started.is_err() {
        log("platform: the window-closed watch did not start");
    }
}

/// SDL's events: the window's closing, and the keys the menus take.
pub fn pump() {
    restore_if_asked();
    #[cfg(feature = "remote")]
    crate::remote::poll();
    while let Some(event) = next_event() {
        match event {
            Event::Quit if HOLD_QUIT.load(std::sync::atomic::Ordering::Relaxed) => {
                log("platform: the window was closed during the copy: quitting at its end");
                QUIT_ASKED.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            Event::Quit => {
                log("platform: the window was closed: the quit switch is set");
                FLAGS.quitting.set(1);
                end_if_not_left();
            }
            // As the window's procedure on `WM_ACTIVATE`: the input thread reads the devices
            // (and does not sleep a second a tick) while it is 1.
            Event::Focus(focused) => FLAGS.window_focused.set(focused as u32),
            Event::Key { scancode, down } => {
                let key = game_key(scancode);
                if key == 0 {
                    continue;
                }
                // The key-down table starts at the pending key and is indexed by the game's
                // key code (0xFF down, as `sar al, 7` leaves it); a key that goes down, or
                // repeats, is the pending key, as the key thread makes it.
                if down {
                    DEVICES.set_key_state(key, 0xFF);
                    crate::input::take_key::pending_key()
                        .store(key, std::sync::atomic::Ordering::SeqCst);
                } else {
                    DEVICES.set_key_state(key, 0);
                }
            }
            Event::Pad { button, down } => pad_button(button, down),
            Event::Other => {}
        }
    }
    let now = u64::from(crate::os::ticks());
    if let Some(action) = lock_menu_pad().tick(now) {
        menu_action(action);
    }
}

/// The menus' side of the gamepads.
fn lock_menu_pad() -> std::sync::MutexGuard<'static, crate::input::menu_pad::MenuPad> {
    static PAD: std::sync::OnceLock<std::sync::Mutex<crate::input::menu_pad::MenuPad>> =
        std::sync::OnceLock::new();
    PAD.get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A gamepad's button, for the menus.
fn pad_button(button: SDL_GamepadButton, down: bool) {
    use crate::input::menu_pad::PadButton as B;
    // The handheld's way out: it has no window to close.
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
        static GUIDE: AtomicBool = AtomicBool::new(false);
        if button == SDL_GAMEPAD_BUTTON_GUIDE {
            GUIDE.store(down, Relaxed);
        } else if button == SDL_GAMEPAD_BUTTON_START && down && GUIDE.load(Relaxed) {
            log("platform: MENU + START: the quit switch is set");
            FLAGS.quitting.set(1);
            end_if_not_left();
            return;
        }
    }
    let button = match button {
        SDL_GAMEPAD_BUTTON_DPAD_UP => B::DpadUp,
        SDL_GAMEPAD_BUTTON_DPAD_DOWN => B::DpadDown,
        SDL_GAMEPAD_BUTTON_DPAD_LEFT => B::DpadLeft,
        SDL_GAMEPAD_BUTTON_DPAD_RIGHT => B::DpadRight,
        SDL_GAMEPAD_BUTTON_SOUTH => B::South,
        SDL_GAMEPAD_BUTTON_EAST => B::East,
        SDL_GAMEPAD_BUTTON_WEST => B::West,
        SDL_GAMEPAD_BUTTON_START => B::Start,
        _ => return,
    };
    let in_race = crate::race::numbers_store::NUMBERS.in_race.get() != 0;
    let now = u64::from(crate::os::ticks());
    if let Some(action) = lock_menu_pad().button(button, down, in_race, now) {
        menu_action(action);
    }
}

/// What a menu takes: the key of the action, as the pending key (not the held-key table: the
/// race's keyboard controls are not the pad's).
fn menu_action(action: crate::input::menu_pad::MenuAction) {
    use crate::input::menu_pad::{MenuAction as A, note_answer};
    use crate::input::yes_no_key::{NO, THIRD, YES};
    let scancode = match action {
        A::Up => SDL_SCANCODE_UP,
        A::Down => SDL_SCANCODE_DOWN,
        A::Left => SDL_SCANCODE_LEFT,
        A::Right => SDL_SCANCODE_RIGHT,
        A::Enter => {
            note_answer(YES);
            SDL_SCANCODE_RETURN
        }
        A::Back => {
            note_answer(NO);
            SDL_SCANCODE_ESCAPE
        }
        A::Third => {
            note_answer(THIRD);
            return;
        }
        A::LeaveRace | A::LeaveRaceEnd => {
            let state = if action == A::LeaveRace { 0xFF } else { 0 };
            DEVICES.set_key_state(game_key(SDL_SCANCODE_ESCAPE.0), state);
            return;
        }
    };
    let key = game_key(scancode.0);
    crate::input::take_key::pending_key().store(key, std::sync::atomic::Ordering::SeqCst);
}

/// SDL's scan code (USB's numbering) as the game's key code: DirectInput's for
/// 1 .. 0x58, and the key thread's (`sub_407AC2`) own numbers for the extended keys it
/// looks at (0x9C .. 0xDD become 0x60 .. 0x7A). 0: a key the game does not know.
fn game_key(scancode: i32) -> u8 {
    const LETTERS: [u8; 26] = [
        0x1E, 0x30, 0x2E, 0x20, 0x12, 0x21, 0x22, 0x23, 0x17, 0x24, 0x25, 0x26, 0x32, 0x31, 0x18,
        0x19, 0x10, 0x13, 0x1F, 0x14, 0x16, 0x2F, 0x11, 0x2D, 0x15, 0x2C,
    ];
    const KEYPAD: [u8; 10] = [0x4F, 0x50, 0x51, 0x4B, 0x4C, 0x4D, 0x47, 0x48, 0x49, 0x52];
    match scancode {
        4..=29 => LETTERS[(scancode - 4) as usize],
        30..=38 => (scancode - 30 + 2) as u8,    // 1 .. 9
        39 => 0x0B,                              // 0
        40 => 0x1C,                              // return
        41 => 0x01,                              // escape
        42 => 0x0E,                              // backspace
        43 => 0x0F,                              // tab
        44 => 0x39,                              // space
        45 => 0x0C,                              // minus
        46 => 0x0D,                              // equals
        47 => 0x1A,                              // [
        48 => 0x1B,                              // ]
        49 | 50 => 0x2B,                         // backslash
        51 => 0x27,                              // ;
        52 => 0x28,                              // '
        53 => 0x29,                              // `
        54 => 0x33,                              // ,
        55 => 0x34,                              // .
        56 => 0x35,                              // /
        57 => 0x3A,                              // caps lock
        58..=67 => (scancode - 58 + 0x3B) as u8, // F1 .. F10
        68 => 0x57,                              // F11
        69 => 0x58,                              // F12
        71 => 0x46,                              // scroll lock
        73 => 0x6D,                              // insert
        74 => 0x65,                              // home
        75 => 0x67,                              // page up
        76 => 0x6E,                              // delete
        77 => 0x6A,                              // end
        78 => 0x6C,                              // page down
        79 => 0x69,                              // right
        80 => 0x68,                              // left
        81 => 0x6B,                              // down
        82 => 0x66,                              // up
        83 => 0x45,                              // num lock
        84 => 0x62,                              // keypad /
        85 => 0x37,                              // keypad *
        86 => 0x4A,                              // keypad -
        87 => 0x4E,                              // keypad +
        88 => 0x60,                              // keypad enter
        89..=97 => KEYPAD[(scancode - 89) as usize],
        98 => 0x52,  // keypad 0
        99 => 0x53,  // keypad .
        100 => 0x56, // the key beside left shift
        101 => 0x7A, // menu
        224 => 0x1D, // left ctrl
        225 => 0x2A, // left shift
        226 => 0x38, // left alt
        227 => 0x78, // left windows
        228 => 0x61, // right ctrl
        229 => 0x36, // right shift
        230 => 0x64, // right alt (AltGr)
        231 => 0x79, // right windows
        _ => 0,
    }
}
