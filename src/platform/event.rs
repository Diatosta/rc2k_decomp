//! SDL's events decoded into the few kinds the program looks at: the window closing, the focus,
//! keys, gamepad buttons and sticks, joysticks, typed text and fingers. The window's own keys
//! (Alt+Enter) and the controllers' bookkeeping (a pad added or removed, which controls the
//! player used last) are done here; what the events mean to the game is `shell::events`'.

use crate::platform::ffi::sdl;
use sdl3_sys::everything::*;

/// A joystick's number as the system gives it.
pub type JoystickId = SDL_JoystickID;

/// A stick's direction, for the menus: the left stick's two axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StickAxis {
    X,
    Y,
}

/// A gamepad button by what it is, whatever the pad's make.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadButton {
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    South,
    East,
    West,
    North,
    LeftShoulder,
    RightShoulder,
    Back,
    Start,
    /// The button in the middle (a handheld's MENU).
    Guide,
    LeftStick,
    RightStick,
    /// A button the program does not use.
    Other,
}

impl PadButton {
    fn from_sdl(button: SDL_GamepadButton) -> Self {
        match button {
            SDL_GAMEPAD_BUTTON_DPAD_UP => Self::DpadUp,
            SDL_GAMEPAD_BUTTON_DPAD_DOWN => Self::DpadDown,
            SDL_GAMEPAD_BUTTON_DPAD_LEFT => Self::DpadLeft,
            SDL_GAMEPAD_BUTTON_DPAD_RIGHT => Self::DpadRight,
            SDL_GAMEPAD_BUTTON_SOUTH => Self::South,
            SDL_GAMEPAD_BUTTON_EAST => Self::East,
            SDL_GAMEPAD_BUTTON_WEST => Self::West,
            SDL_GAMEPAD_BUTTON_NORTH => Self::North,
            SDL_GAMEPAD_BUTTON_LEFT_SHOULDER => Self::LeftShoulder,
            SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER => Self::RightShoulder,
            SDL_GAMEPAD_BUTTON_BACK => Self::Back,
            SDL_GAMEPAD_BUTTON_START => Self::Start,
            SDL_GAMEPAD_BUTTON_GUIDE => Self::Guide,
            SDL_GAMEPAD_BUTTON_LEFT_STICK => Self::LeftStick,
            SDL_GAMEPAD_BUTTON_RIGHT_STICK => Self::RightStick,
            _ => Self::Other,
        }
    }

    /// The button's number in the order the profiles number a gamepad's buttons (see
    /// `input::PAD_BUTTONS`), if it has one.
    pub fn profile_number(self) -> Option<u8> {
        let sdl = match self {
            Self::South => SDL_GAMEPAD_BUTTON_SOUTH,
            Self::East => SDL_GAMEPAD_BUTTON_EAST,
            Self::West => SDL_GAMEPAD_BUTTON_WEST,
            Self::North => SDL_GAMEPAD_BUTTON_NORTH,
            Self::LeftShoulder => SDL_GAMEPAD_BUTTON_LEFT_SHOULDER,
            Self::RightShoulder => SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER,
            Self::Back => SDL_GAMEPAD_BUTTON_BACK,
            Self::Start => SDL_GAMEPAD_BUTTON_START,
            Self::LeftStick => SDL_GAMEPAD_BUTTON_LEFT_STICK,
            Self::RightStick => SDL_GAMEPAD_BUTTON_RIGHT_STICK,
            _ => return None,
        };
        crate::platform::input::PAD_BUTTONS
            .iter()
            .position(|&b| b == sdl)
            .map(|n| n as u8)
    }
}

/// Scan codes (USB's numbering) of the keys the program sends itself.
pub mod scancode {
    use sdl3_sys::everything::*;

    pub const UP: i32 = SDL_SCANCODE_UP.0;
    pub const DOWN: i32 = SDL_SCANCODE_DOWN.0;
    pub const LEFT: i32 = SDL_SCANCODE_LEFT.0;
    pub const RIGHT: i32 = SDL_SCANCODE_RIGHT.0;
    pub const RETURN: i32 = SDL_SCANCODE_RETURN.0;
    pub const ESCAPE: i32 = SDL_SCANCODE_ESCAPE.0;
    pub const TAB: i32 = SDL_SCANCODE_TAB.0;
}

/// An event the program looks at.
pub enum Event {
    Quit,
    Focus(bool),
    /// A window's size changed.
    Resized,
    /// Ctrl+F11: the debug window opens or closes.
    ToggleDebugUi,
    Key {
        scancode: i32,
        down: bool,
    },
    Pad {
        id: JoystickId,
        button: PadButton,
        down: bool,
    },
    Stick {
        axis: StickAxis,
        value: i16,
    },
    /// The right stick's vertical axis (a page that scrolls reads it).
    ScrollStick(i32),
    /// A joystick that is not a gamepad (a wheel): its hat, buttons and first axis.
    Joystick {
        id: JoystickId,
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

/// SDL's next event, if there is one. `overlay_takes` is asked first, for every event except
/// the window's own keys: when it takes the event (the debug window's keyboard and mouse) the
/// game does not see it.
pub fn next(overlay_takes: impl FnOnce(&sdl::PolledEvent) -> bool) -> Option<Event> {
    let event = sdl::poll_event()?;
    // Ctrl+F11, in the race and in the menus (which never run `input::update`), and
    // whatever has the UI's keyboard.
    if event.is_fresh_ctrl_key_down(SDL_SCANCODE_F11) {
        return Some(Event::ToggleDebugUi);
    }
    // Alt+Enter: the window to full screen and back (the graphics page's "screen" row).
    if event.is_fresh_alt_key_down(SDL_SCANCODE_RETURN) {
        crate::platform::window_mode::set(crate::platform::window_mode::toggled());
        crate::platform::apply_window_mode();
        return Some(Event::Other);
    }
    // The debug UI's first: what it takes the game does not see.
    if overlay_takes(&event) {
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
            Event::Pad {
                id,
                button: PadButton::from_sdl(button),
                down,
            }
        } else if let Some((axis, value)) = event.gamepad_axis() {
            // A stick pushed well off its rest, or a trigger pressed, is the controller used.
            #[cfg(phone)]
            if value.unsigned_abs() > 16000
                && let Some(id) = event.gamepad_axis_joystick()
            {
                crate::platform::touch::pad_used(id);
            }
            let axis = match axis {
                SDL_GAMEPAD_AXIS_LEFTX => StickAxis::X,
                SDL_GAMEPAD_AXIS_LEFTY => StickAxis::Y,
                SDL_GAMEPAD_AXIS_RIGHTY => return Some(Event::ScrollStick(value.into())),
                _ => return Some(Event::Other),
            };
            Event::Stick { axis, value }
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
