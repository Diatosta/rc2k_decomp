//! Module input's owner (step 11, owned state): the input stores in one struct, reached
//! through `input()` by the code that does not take it as a parameter.

use crate::input::guids::InputGuids;
use crate::input::store::{Devices, ForceFeedback, Keyboard, Mouse};

/// The input's state.
pub struct Input {
    /// The input devices, the key tables and the settings kept with them.
    pub devices: Devices,
    /// The keyboard layout and the typed-key state.
    pub keyboard: Keyboard,
    /// The mouse position and buttons.
    pub mouse: Mouse,
    /// The force feedback's clock and effects.
    pub force_feedback: ForceFeedback,
    /// The device and axis GUIDs.
    pub guids: InputGuids,
}

/// The one instance. Reached through `input()`; `moved` registers its stores' places.
pub static INPUT: Input = Input {
    devices: Devices::new(),
    keyboard: Keyboard::new(),
    mouse: Mouse::new(),
    force_feedback: ForceFeedback::new(),
    guids: InputGuids::new(),
};

/// The input's state, for a caller that does not take it as a parameter yet.
pub fn input() -> &'static Input {
    &INPUT
}
