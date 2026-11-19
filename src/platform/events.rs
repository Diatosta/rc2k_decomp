//! SDL's events under the game: closing the window sets the quit switch, and the keys and
//! buttons the menus know become the game's pending key.

use crate::logfile::program_log::log;
use crate::platform::ffi::sdl;
use rc2k_core::input::profile::MenuControl;
use sdl3_sys::everything::*;

/// An SDL event the program looks at.
enum Event {
    Quit,
    Focus(bool),
    /// A window's size changed.
    Resized,
    Key {
        scancode: i32,
        down: bool,
    },
    Pad {
        id: SDL_JoystickID,
        button: SDL_GamepadButton,
        down: bool,
    },
    Stick {
        axis: crate::input::menu_pad::StickAxis,
        value: i32,
    },
    /// The right stick's vertical axis (a page that scrolls reads it).
    ScrollStick(i32),
    /// A joystick that is not a gamepad (a wheel): its hat, buttons and first axis.
    Joystick {
        id: SDL_JoystickID,
        input: sdl::JoystickInput,
    },
    /// Typed text (UTF-8).
    Text(String),
    /// A finger on the touch screen.
    #[cfg(touch_screen)]
    Finger {
        id: u64,
        phase: rc2k_core::input::menu_touch::Phase,
        x: f32,
        y: f32,
    },
    Other,
}

/// A finger's change as an event (touch screens only).
#[cfg(touch_screen)]
fn finger_event(event: &sdl::PolledEvent) -> Option<Event> {
    let (id, phase, x, y) = event.finger()?;
    Some(Event::Finger { id, phase, x, y })
}

#[cfg(not(touch_screen))]
fn finger_event(_: &sdl::PolledEvent) -> Option<Event> {
    None
}

/// SDL's next event, if there is one.
fn next_event() -> Option<Event> {
    let event = sdl::poll_event()?;
    // Ctrl+F11, in the race and in the menus (which never run `input::update`), and
    // whatever has the UI's keyboard.
    if event.is_fresh_ctrl_key_down(SDL_SCANCODE_F11) {
        crate::debug::overlay::toggle_ui();
        return Some(Event::Other);
    }
    // Alt+Enter: the window to full screen and back (the graphics page's "screen" row).
    if event.is_fresh_alt_key_down(SDL_SCANCODE_RETURN) {
        crate::platform::window_mode::set(crate::platform::window_mode::toggled());
        crate::platform::apply_window_mode();
        return Some(Event::Other);
    }
    // The debug UI's first (`debug::imgui_sdl`): what it takes the game does not see.
    if crate::debug::imgui_sdl::event(event.as_sdl()) {
        return Some(Event::Other);
    }
    let kind = event.kind();
    // A window's close button: the OpenGL backend's window may be the only one shown, and SDL
    // sends its quit only when the last of all its windows closes.
    Some(
        if kind == SDL_EVENT_QUIT || kind == SDL_EVENT_WINDOW_CLOSE_REQUESTED {
            Event::Quit
        } else if kind == SDL_EVENT_WINDOW_RESIZED {
            Event::Resized
        } else if kind == SDL_EVENT_WINDOW_FOCUS_GAINED {
            Event::Focus(true)
        } else if kind == SDL_EVENT_WINDOW_FOCUS_LOST || kind == SDL_EVENT_WILL_ENTER_BACKGROUND {
            // A phone's app sent to the background (the system takes the GPU from it).
            Event::Focus(false)
        } else if kind == SDL_EVENT_DID_ENTER_FOREGROUND {
            Event::Focus(true)
        } else if kind == SDL_EVENT_JOYSTICK_ADDED {
            #[cfg(phone)]
            crate::platform::touch::pad_added(event.joystick_id());
            crate::platform::input::pad_added(event.joystick_id());
            Event::Other
        } else if kind == SDL_EVENT_JOYSTICK_REMOVED {
            #[cfg(phone)]
            crate::platform::touch::pad_removed(event.joystick_id());
            crate::platform::input::pad_removed(event.joystick_id());
            Event::Other
        } else if let Some(finger) = finger_event(&event) {
            finger
        } else if let Some((id, button, down)) = event.gamepad_button() {
            #[cfg(phone)]
            if down {
                crate::platform::touch::pad_used(id);
            }
            Event::Pad { id, button, down }
        } else if let Some((axis, value)) = event.gamepad_axis() {
            // A stick pushed well off its rest, or a trigger pressed, is the controller used.
            #[cfg(phone)]
            if value.unsigned_abs() > 16000
                && let Some(id) = event.gamepad_axis_joystick()
            {
                crate::platform::touch::pad_used(id);
            }
            use crate::input::menu_pad::StickAxis;
            let axis = match axis {
                SDL_GAMEPAD_AXIS_LEFTX => StickAxis::X,
                SDL_GAMEPAD_AXIS_LEFTY => StickAxis::Y,
                SDL_GAMEPAD_AXIS_RIGHTY => return Some(Event::ScrollStick(value.into())),
                _ => return Some(Event::Other),
            };
            Event::Stick {
                axis,
                value: value.into(),
            }
        } else if let Some((id, input)) = event.joystick_input() {
            Event::Joystick { id, input }
        } else if let Some(text) = event.text() {
            Event::Text(text)
        } else if let Some((scancode, down)) = event.key() {
            Event::Key { scancode, down }
        } else {
            Event::Other
        },
    )
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
        crate::platform::input_window(screen.window).restore_if_minimised();
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
pub fn hold_quit(system: &crate::system::owner::System, on: bool) {
    use std::sync::atomic::Ordering::Relaxed;
    HOLD_QUIT.store(on, Relaxed);
    if !on && QUIT_ASKED.swap(false, Relaxed) {
        log("platform: the copy has ended: the quit switch is set");
        system.session.flags.quitting.set(true);
        end_if_not_left();
    }
}

/// After the window's closing: the menus, races and films look at the quit switch and leave (fixed
/// bug: "Closing the window leaves the game running"). A safety net for a loop that still does not:
/// if the program is running `LEAVING_GRACE` after the first close, it ends, saving nothing: the
/// game's thread may be in the tables a save reads.
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
pub fn pump(o: crate::owners::Owners<'static>) {
    let (input, system) = (o.input, o.system);
    restore_if_asked();
    // Controllers plugged in or out: the players given the ones they chose.
    crate::input::device_ids::after_listing(o.menus, input);
    #[cfg(feature = "remote")]
    crate::remote::poll(&o, &o.race.course);
    // The browser's gamepads copied into SDL's virtual ones (the page's API is not on this thread).
    #[cfg(web)]
    {
        crate::platform::web::run_queued_calls();
        crate::platform::web_pads::poll();
        crate::platform::web::follow_full_screen();
    }
    while let Some(event) = next_event() {
        match event {
            Event::Quit if HOLD_QUIT.load(std::sync::atomic::Ordering::Relaxed) => {
                log("platform: the window was closed during the copy: quitting at its end");
                QUIT_ASKED.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            Event::Quit => {
                log("platform: the window was closed: the quit switch is set");
                system.session.flags.quitting.set(true);
                end_if_not_left();
            }
            // As the window's procedure on `WM_ACTIVATE`: the input thread reads the devices
            // (and does not sleep a second a tick) while it is 1.
            Event::Resized => {
                #[cfg(not(handheld))]
                crate::platform::window_follow::note_resize(
                    u64::from(crate::platform::os::ticks()),
                );
            }
            Event::Focus(focused) => {
                system.session.flags.window_focused.set(focused as u32);
                // Fingers on the screen when the app is sent away or comes back may never
                // be lifted for it (a system gesture took them).
                #[cfg(touch_screen)]
                crate::platform::touch_menu::release_all();
            }
            Event::Text(text) => crate::input::typed_text::typed_by_keyboard(&text),
            Event::Key { scancode, down } => {
                if down && matches!(scancode, 40 | 88) {
                    crate::input::typed_text::note_confirm(false);
                }
                let key = game_key(scancode);
                if key == 0 {
                    continue;
                }
                // The key-down table starts at the pending key and is indexed by the game's key
                // code (0xFF: down); a key that goes down, or repeats, is the pending key, as the
                // key thread makes it.
                if down {
                    input.devices.set_key_state(key, 0xFF);
                    crate::input::take_key::pending_key(input)
                        .store(key, std::sync::atomic::Ordering::SeqCst);
                } else {
                    input.devices.set_key_state(key, 0);
                }
            }
            Event::Pad { id, button, down } => {
                pad_bound_controls(input, id, button, down, driving(o.race));
                pad_button(driving(o.race), input, system, id, button, down);
            }
            // In a race the stick steers (the game's own devices): only the menus and the
            // replay viewer take it.
            Event::ScrollStick(value) => input.note_scroll_stick(value),
            Event::Stick { axis, value } => {
                if !driving(o.race) {
                    let now = u64::from(crate::platform::os::ticks());
                    if let Some(action) = lock_menu_pad().stick(axis, value, now) {
                        menu_action(input, action);
                    }
                }
            }
            // In a race a joystick's axes and hat drive the car; its menu buttons still answer the
            // race's questions (leave the stage?), as a gamepad's do.
            Event::Joystick { id, input: what } => {
                // A gamepad's joystick sends these beside its gamepad events, numbered as the
                // device's own report has it: its menu controls come from the gamepad events
                // (`pad_bound_controls`), numbered as the profiles are.
                if crate::platform::ffi::sdl_input::is_gamepad(id) {
                    continue;
                }
                // A wheel's enter and back buttons end what ends on any key, in a race too.
                if let sdl::JoystickInput::Button(1..=2, true) = what {
                    input.note_pad_press(false);
                }
                let driving = driving(o.race);
                if !driving || matches!(what, sdl::JoystickInput::Button(..)) {
                    joystick_in_menus(input, id, what, driving);
                }
            }
            #[cfg(touch_screen)]
            Event::Finger { id, phase, x, y } => {
                let now = u64::from(crate::platform::os::ticks());
                touch_finger(&o, id, phase, x, y, now);
            }
            Event::Other => {}
        }
    }
    let now = u64::from(crate::platform::os::ticks());
    #[cfg(phone)]
    {
        // Player 1's controls changed hands (a controller used, a finger down in a race, a
        // controller gone): the game's device for the car follows.
        if crate::platform::touch::take_source_change() {
            crate::input::device_ids::source_changed(o.menus, input, o.race, driving(o.race));
        }
        crate::platform::touch::update(touch_controls_live(o.race), now);
    }
    #[cfg(touch_screen)]
    touch_tick(&o, now);
    #[cfg(not(handheld))]
    crate::platform::window_follow::follow(o, now);
    if let Some(action) = lock_menu_pad().tick(now) {
        menu_action(input, action);
    }
    sync_text_input();
}

/// SDL's text input on while a text field is edited and off after (a screen keyboard, where
/// the system has one, shows with it).
fn sync_text_input() {
    let screen = crate::platform::SCREEN.lock();
    let window = screen
        .as_ref()
        .ok()
        .and_then(|screen| screen.as_ref())
        .map(|screen| crate::platform::input_window(screen.window));
    let has_keyboard = || window.is_some_and(super::ffi::sdl::Window::has_screen_keyboard);
    if let Some(on) = crate::input::typed_text::sync(has_keyboard)
        && let Some(window) = window
    {
        window.set_text_input(on);
    }
    if let Some(window) = window
        && crate::input::typed_text::text_mode()
    {
        // Fields that do not say where they are sit about the middle of the screen.
        window.set_text_input_area(crate::input::typed_text::field_y().unwrap_or(300));
    }
}

/// Whether a race is being driven: the pad and the wheel are then the player's controls. In the
/// replay viewer a race is loaded, but its list is a menu: the pad works it as it works the menus.
pub(crate) fn driving(race: &crate::race::owner::Race) -> bool {
    race.course.numbers.get().in_race.is_set() && race.shared.replay_file().hold_car_revs.get() == 0
}

/// Whether the race's own touch controls are live: a race is driven and no pause or question
/// (`drive::pause_until_key`, `drive::quit_question`, which set `scene_switch`) waits for an
/// answer. While one does, a finger is the menus' gestures: a tap is Enter (any key, yes), a
/// double tap Escape (no).
#[cfg(touch_screen)]
pub(crate) fn touch_controls_live(race: &crate::race::owner::Race) -> bool {
    driving(race) && race.shared.vars.get().scene_switch.get() == 0
}

/// What the race waits for while its touch controls are off: the quit question's yes and no, or
/// the plain pause's key. A finger then presses the answer buttons drawn over the frame (and
/// nothing else answers: a tap elsewhere does nothing).
#[cfg(phone)]
pub(crate) fn touch_asked(
    race: &crate::race::owner::Race,
) -> Option<rc2k_core::input::touch::Asked> {
    use rc2k_core::input::touch::Asked;
    if !driving(race) || race.shared.vars.get().scene_switch.get() == 0 {
        return None;
    }
    Some(if crate::drive::quit_question::asking() {
        Asked::YesNo
    } else {
        Asked::Resume
    })
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
#[cfg_attr(
    not(handheld),
    allow(
        unused_variables,
        reason = "only the handheld's way out sets the quit switch"
    )
)]
fn pad_button(
    driving: bool,
    input: &crate::input::owner::Input,
    system: &crate::system::owner::System,
    pad: SDL_JoystickID,
    button: SDL_GamepadButton,
    down: bool,
) {
    use crate::input::menu_pad::PadButton as B;
    // The handheld's way out: it has no window to close.
    #[cfg(handheld)]
    {
        use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
        static GUIDE: AtomicBool = AtomicBool::new(false);
        if button == SDL_GAMEPAD_BUTTON_GUIDE {
            GUIDE.store(down, Relaxed);
        } else if button == SDL_GAMEPAD_BUTTON_START && down && GUIDE.load(Relaxed) {
            log("platform: MENU + START: the quit switch is set");
            system.session.flags.quitting.set(true);
            end_if_not_left();
            return;
        }
    }
    // The on-screen keyboard's own buttons: Start is done, Back cancels.
    if down
        && (button == SDL_GAMEPAD_BUTTON_START || button == SDL_GAMEPAD_BUTTON_BACK)
        && crate::input::typed_text::osk().is_some()
    {
        let asks = if button == SDL_GAMEPAD_BUTTON_START {
            crate::input::typed_text::Asks::Done
        } else {
            crate::input::typed_text::Asks::Cancel
        };
        osk_asks(input, asks);
        return;
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
        // In a race the shoulders may be bound to the car's controls.
        SDL_GAMEPAD_BUTTON_LEFT_SHOULDER if !driving => B::LeftShoulder,
        SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER if !driving => B::RightShoulder,
        _ => return,
    };
    // A control the pad's profile binds is taken from its bound button instead.
    if let Some(control) = menu_control_of(button)
        && menu_map(input, pad).is_some_and(|map| map.is_bound(control))
    {
        return;
    }
    if down && matches!(button, B::South | B::East | B::Start) {
        input.note_pad_press(button == B::Start);
    }
    if down {
        match button {
            B::LeftShoulder => input.note_shoulder(1),
            B::RightShoulder => input.note_shoulder(2),
            _ => {}
        }
    }
    let in_race = driving;
    let now = u64::from(crate::platform::os::ticks());
    if let Some(action) = lock_menu_pad().button(button, down, in_race, now) {
        menu_action(input, action);
    }
}

/// A joystick that is not a gamepad (a wheel) in the menus: its hat is the pad's cross (its axes
/// move nothing: a wheel turned while choosing would wander through the items), its second button
/// enters and its third goes back. Each button press is logged, so a
/// wheel whose buttons are numbered otherwise can be told.
fn joystick_in_menus(
    input: &crate::input::owner::Input,
    id: SDL_JoystickID,
    what: sdl::JoystickInput,
    in_race: bool,
) {
    use crate::input::menu_pad::PadButton;
    use sdl::JoystickInput as J;
    let now = u64::from(crate::platform::os::ticks());
    bound_menu_controls(input, id, what, in_race);
    let map = menu_map(input, id);
    let given_up = |control| map.is_some_and(|map| map.is_bound(control));
    let action = match what {
        J::Hat(0, bits) => {
            let bits = bits & !map.map_or(0, |map| map.moves_bound());
            lock_menu_pad().hat(bits, now)
        }
        J::Hat(..) => None,
        J::Button(n, down) => {
            // Logged so a player can tell which number a button is; not in a race, where the
            // buttons shift gears and the like.
            if down && !in_race {
                log(&format!(
                    "platform: {} button {} in the menus{}",
                    crate::platform::ffi::sdl_input::joystick_name_for_id(id, "a joystick"),
                    n + 1,
                    match n {
                        1 => " (enter)",
                        2 => " (back)",
                        _ => "",
                    }
                ));
            }
            let button = match n {
                // The second and third buttons: cross and circle on a wheel with a PlayStation
                // pad's faces (square is the first).
                1 if !given_up(MenuControl::Enter) => PadButton::South,
                2 if !given_up(MenuControl::Back) => PadButton::East,
                _ => return,
            };
            lock_menu_pad().button(button, down, in_race, now)
        }
    };
    if let Some(action) = action {
        menu_action(input, action);
    }
}

/// The menu bindings of the controller `id` (see `input::profiles::menu_map_of`).
fn menu_map(
    input: &crate::input::owner::Input,
    id: SDL_JoystickID,
) -> Option<rc2k_core::input::profile::MenuMap> {
    let name = crate::platform::ffi::sdl_input::joystick_name_for_id(id, "");
    crate::input::profiles::menu_map_of(input, &name)
}

/// The menu control a gamepad button stands for by default (the pad's Start, which leaves a
/// race, and the shoulders' use in a race are not bindable).
fn menu_control_of(button: crate::input::menu_pad::PadButton) -> Option<MenuControl> {
    use crate::input::menu_pad::PadButton as B;
    Some(match button {
        B::South => MenuControl::Enter,
        B::East => MenuControl::Back,
        B::West => MenuControl::Third,
        B::LeftShoulder | B::RightShoulder => MenuControl::Tab,
        B::DpadUp => MenuControl::Up,
        B::DpadDown => MenuControl::Down,
        B::DpadLeft => MenuControl::Left,
        B::DpadRight => MenuControl::Right,
        B::Start => return None,
    })
}

/// A gamepad's button, as the profile numbers it (`PAD_BUTTONS`, the cross as hat 0), to the menu
/// controls its profile binds: the learning reads the pad through that numbering, so a raw joystick
/// number (which differs by model: a DualSense's L3 / R3 among them) would miss.
fn pad_bound_controls(
    input: &crate::input::owner::Input,
    id: SDL_JoystickID,
    button: SDL_GamepadButton,
    down: bool,
    in_race: bool,
) {
    use std::collections::HashMap;
    static CROSS: std::sync::Mutex<Option<HashMap<SDL_JoystickID, u8>>> =
        std::sync::Mutex::new(None);
    let bit = match button {
        SDL_GAMEPAD_BUTTON_DPAD_UP => 1,
        SDL_GAMEPAD_BUTTON_DPAD_RIGHT => 2,
        SDL_GAMEPAD_BUTTON_DPAD_DOWN => 4,
        SDL_GAMEPAD_BUTTON_DPAD_LEFT => 8,
        _ => 0,
    };
    let what = if bit != 0 {
        let mut cross = CROSS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let bits = cross
            .get_or_insert_with(HashMap::new)
            .entry(id)
            .or_insert(0);
        if down {
            *bits |= bit;
        } else {
            *bits &= !bit;
        }
        sdl::JoystickInput::Hat(0, *bits)
    } else {
        match crate::platform::input::PAD_BUTTONS
            .iter()
            .position(|&b| b == button)
        {
            Some(n) => sdl::JoystickInput::Button(n as u8, down),
            None => return,
        }
    };
    bound_menu_controls(input, id, what, in_race);
}

/// The menu controls the controller's profile binds to what moved: each is pressed or let go as
/// the pad's button that stands for it.
fn bound_menu_controls(
    input: &crate::input::owner::Input,
    id: SDL_JoystickID,
    what: sdl::JoystickInput,
    in_race: bool,
) {
    use crate::input::menu_pad::PadButton as B;
    let Some(map) = menu_map(input, id) else {
        return;
    };
    let now = u64::from(crate::platform::os::ticks());
    let mut pressed: Vec<(MenuControl, bool)> = Vec::new();
    match what {
        sdl::JoystickInput::Button(n, down) => {
            if n < 32 {
                pressed.extend(map.on_button(n).map(|c| (c, down)));
            }
        }
        sdl::JoystickInput::Hat(hat, bits) => {
            let (newly, gone) = lock_menu_pad().bound_hat_changes(hat, bits);
            for (control, bit) in map.on_hat(hat) {
                if newly & bit != 0 {
                    pressed.push((control, true));
                } else if gone & bit != 0 {
                    pressed.push((control, false));
                }
            }
        }
    }
    for (control, down) in pressed {
        let button = match control {
            MenuControl::Enter => B::South,
            MenuControl::Back => B::East,
            MenuControl::Third => B::West,
            MenuControl::Tab => B::LeftShoulder,
            MenuControl::Up => B::DpadUp,
            MenuControl::Down => B::DpadDown,
            MenuControl::Left => B::DpadLeft,
            MenuControl::Right => B::DpadRight,
        };
        if down && matches!(button, B::South | B::East) {
            input.note_pad_press(false);
        }
        if let Some(action) = lock_menu_pad().button(button, down, in_race, now) {
            menu_action(input, action);
        }
    }
}

/// What a menu takes: the key of the action, as the pending key (not the held-key table: the
/// race's keyboard controls are not the pad's).
pub(crate) fn menu_action(
    input: &crate::input::owner::Input,
    action: crate::input::menu_pad::MenuAction,
) {
    use crate::input::menu_pad::{MenuAction as A, note_answer};
    use crate::input::yes_no_key::{NO, THIRD, YES};
    // The on-screen keyboard takes the moves and the south, east and west buttons.
    if crate::input::typed_text::osk().is_some() {
        let step = match action {
            A::Up => Some((0, -1)),
            A::Down => Some((0, 1)),
            A::Left => Some((-1, 0)),
            A::Right => Some((1, 0)),
            _ => None,
        };
        if let Some((dx, dy)) = step {
            crate::input::typed_text::osk_step(dx, dy);
            return;
        }
        match action {
            A::Enter => {
                if let Some(asks) = crate::input::typed_text::osk_press() {
                    osk_asks(input, asks);
                }
                return;
            }
            // The east button (B on an Xbox pad) cancels the field, the west one (X) deletes.
            A::Back => {
                osk_asks(input, crate::input::typed_text::Asks::Cancel);
                return;
            }
            A::Third => {
                osk_asks(input, crate::input::typed_text::Asks::Backspace);
                return;
            }
            _ => {}
        }
    }
    if action == A::Enter {
        crate::input::typed_text::note_confirm(true);
    }
    let scancode = match action {
        A::Up => SDL_SCANCODE_UP,
        A::Down => SDL_SCANCODE_DOWN,
        A::Left => SDL_SCANCODE_LEFT,
        A::Right => SDL_SCANCODE_RIGHT,
        A::Enter => {
            note_answer(input, YES);
            SDL_SCANCODE_RETURN
        }
        A::Back => {
            note_answer(input, NO);
            SDL_SCANCODE_ESCAPE
        }
        A::Third => {
            note_answer(input, THIRD);
            // Also a key the menus ignore, so that what waits for any key (the films, the
            // messages, the pauses) takes this button as it takes the others.
            crate::input::take_key::pending_key(input).store(
                rc2k_core::menu::keys::IGNORED,
                std::sync::atomic::Ordering::SeqCst,
            );
            return;
        }
        A::Tab => SDL_SCANCODE_TAB,
        A::LeaveRace | A::LeaveRaceEnd => {
            let state = if action == A::LeaveRace { 0xFF } else { 0 };
            input
                .devices
                .set_key_state(game_key(SDL_SCANCODE_ESCAPE.0), state);
            return;
        }
    };
    let key = game_key(scancode.0);
    crate::input::take_key::pending_key(input).store(key, std::sync::atomic::Ordering::SeqCst);
}

/// SDL's scan code (USB's numbering) as the game's key code: DirectInput's for
/// 1 .. 0x58, and the key thread's own numbers for the extended keys it
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
        41 | 270 => 0x01,                        // escape, and a phone's back button
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

/// What the on-screen keyboard asks, as the keys the field already takes.
fn osk_asks(input: &crate::input::owner::Input, asks: crate::input::typed_text::Asks) {
    use crate::input::typed_text::Asks;
    let key = match asks {
        Asks::Nothing => return,
        Asks::Backspace => 0x0E,
        Asks::Done => 0x1C,
        Asks::Cancel => 0x01,
    };
    if asks == Asks::Backspace {
        crate::input::typed_text::osk_erased();
    } else {
        crate::input::typed_text::osk_close();
    }
    crate::input::take_key::pending_key(input).store(key, std::sync::atomic::Ordering::SeqCst);
}

/// A single tap waiting for its double-tap window is let through once the window is over.
#[cfg(touch_screen)]
fn touch_tick(o: &crate::owners::Owners<'static>, now: u64) {
    let window = {
        let screen = crate::platform::SCREEN.lock();
        screen
            .as_ref()
            .ok()
            .and_then(|screen| screen.as_ref())
            .map(|screen| crate::platform::input_window(screen.window))
    };
    #[cfg(phone)]
    if touch_asked(o.race).is_some() {
        return;
    }
    if let Some(window) = window
        && !touch_controls_live(o.race)
    {
        crate::platform::touch_menu::tick(o, window, now);
    }
}

/// A finger on the touch screen: in the menus a gesture or a tap (`platform::touch_menu`).
#[cfg(touch_screen)]
fn touch_finger(
    o: &crate::owners::Owners<'static>,
    id: u64,
    phase: rc2k_core::input::menu_touch::Phase,
    x: f32,
    y: f32,
    now: u64,
) {
    let window = {
        let screen = crate::platform::SCREEN.lock();
        screen
            .as_ref()
            .ok()
            .and_then(|screen| screen.as_ref())
            .map(|screen| crate::platform::input_window(screen.window))
    };
    let Some(window) = window else {
        // No window to size the gestures by: a lifting still takes its finger off the list.
        if phase == rc2k_core::input::menu_touch::Phase::Up {
            crate::platform::touch_menu::forget(id);
        }
        return;
    };
    if touch_controls_live(o.race) {
        crate::platform::touch_menu::release_all();
        #[cfg(phone)]
        crate::platform::touch::finger(window, id, phase, x, y, now);
        return;
    }
    #[cfg(phone)]
    if let Some(asked) = touch_asked(o.race) {
        crate::platform::touch_menu::release_all();
        let answer = crate::platform::touch::answer_finger(window, asked, id, phase, x, y);
        match answer {
            Some(rc2k_core::input::touch::Answer::Yes) => {
                menu_action(o.input, crate::input::menu_pad::MenuAction::Enter);
            }
            Some(rc2k_core::input::touch::Answer::No) => {
                menu_action(o.input, crate::input::menu_pad::MenuAction::Back);
            }
            None => {}
        }
        return;
    }
    set_viewer_sideways(o.race);
    crate::platform::touch_menu::finger(o, window, id, phase, x, y, now);
}

/// In the replay viewer (a replay is shown, `driving` is false) a sideways drag gives the key on
/// the finger's side, so a drag to the right is Right (the replay goes on), as on the save code
/// page; the menus' carousels keep the finger-follows-content direction. Set only when the
/// viewer is entered or left, so the page that sets it for itself is left alone.
#[cfg(touch_screen)]
fn set_viewer_sideways(race: &crate::race::owner::Race) {
    use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
    static IN_VIEWER: AtomicBool = AtomicBool::new(false);
    let viewing = race.course.numbers.get().in_race.is_set()
        && race.shared.replay_file().hold_car_revs.get() != 0;
    if IN_VIEWER.swap(viewing, Relaxed) != viewing {
        crate::platform::touch_menu::set_direct_sideways(viewing);
    }
}
