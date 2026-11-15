//! Module input's owner: the input stores in one struct, handed by reference to the code that
//! uses them.

use crate::input::guids::InputGuids;
use crate::input::layouts::Layouts;
use crate::input::store::{Devices, ForceFeedback, Keyboard, Mouse};
use crate::session::GameThread;
use std::sync::atomic::AtomicU32;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// The input's state.
pub struct Input {
    /// The input devices, the key tables and the settings kept with them.
    pub devices: Devices,
    /// The force feedback's clock and effects.
    pub force_feedback: ForceFeedback,
    /// The device and axis GUIDs.
    pub guids: InputGuids,
    /// The keyboard layouts the game types with.
    layouts: Mutex<Layouts>,
    /// The answer a gamepad button gave to a yes / no question, for the question to take
    /// (`input::menu_pad::note_answer`).
    pub menu_answer: AtomicU32,
    /// The keyboard and the mouse: only the game's thread reaches them (the input thread
    /// polls the devices, the keys and the mouse come from the window's events).
    main: GameThread<InputMain>,
}

/// The keyboard and the mouse.
pub struct InputMain {
    /// The keyboard layout and the typed-key state.
    pub keyboard: Keyboard,
    /// The mouse position and buttons.
    pub mouse: Mouse,
}

impl Input {
    /// The keyboard layouts.
    pub fn layouts(&self) -> MutexGuard<'_, Layouts> {
        self.layouts.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The keyboard layout and the typed-key state.
    pub fn keyboard(&self) -> &Keyboard {
        &self.main.get().keyboard
    }

    /// The mouse position and buttons.
    pub fn mouse(&self) -> &Mouse {
        &self.main.get().mouse
    }
}

impl Input {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
            devices: Devices::new(),
            force_feedback: ForceFeedback::new(),
            guids: InputGuids::new(),
            layouts: Mutex::new(Layouts::new()),
            menu_answer: AtomicU32::new(crate::input::menu_pad::NO_ANSWER),
            main: GameThread::new(InputMain {
                keyboard: Keyboard::new(),
                mouse: Mouse::new(),
            }),
        }
    }
}
