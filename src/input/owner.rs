//! Module input's owner: the input stores in one struct, reached through `input()` by the code
//! that does not take it as a parameter.

use crate::input::guids::InputGuids;
use crate::input::store::{Devices, ForceFeedback, Keyboard, Mouse};
use crate::session::GameThread;

/// The input's state.
pub struct Input {
    /// The input devices, the key tables and the settings kept with them.
    pub devices: Devices,
    /// The force feedback's clock and effects.
    pub force_feedback: ForceFeedback,
    /// The device and axis GUIDs.
    pub guids: InputGuids,
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
    /// The keyboard layout and the typed-key state.
    pub fn keyboard(&self) -> &Keyboard {
        &self.main.get().keyboard
    }

    /// The mouse position and buttons.
    pub fn mouse(&self) -> &Mouse {
        &self.main.get().mouse
    }
}

/// The one instance. Reached through `input()`; `moved` registers its stores' places.
pub static INPUT: Input = Input {
    devices: Devices::new(),
    force_feedback: ForceFeedback::new(),
    guids: InputGuids::new(),
    main: GameThread::new(InputMain {
        keyboard: Keyboard::new(),
        mouse: Mouse::new(),
    }),
};

/// The input's state, for a caller that does not take it as a parameter yet.
pub fn input() -> &'static Input {
    &INPUT
}
