//! Module input's owner: the input stores in one struct, handed by reference to the code that
//! uses them.

use crate::input::guids::InputGuids;
use crate::input::layouts::Layouts;
use crate::input::store::{Devices, ForceFeedback, Keyboard, Mouse};
use crate::memory::session::GameThread;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
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
    /// How many times the pad's A, B or Start (or a wheel's first two buttons) went down: what a
    /// loop that ends on any key reads for the pad (`pad_pressed_since`).
    pad_presses: AtomicU32,
    /// How many times Start went down (the network wait's way out).
    pad_starts: AtomicU32,
    /// While a controller's binding is learnt and the pad's Back is held: when it went down
    /// (ms, plus one; 0 when it is up).
    pub learn_back_since: std::sync::atomic::AtomicU64,
    /// The controller each of the four players chose, by what it is (`input::device_ids`):
    /// kept while it is not plugged in, so the player has it back when it is.
    pub chosen_devices: Mutex<[Option<rc2k_core::input::device_id::DeviceId>; 4]>,
    /// Each player's profiles, one for every controller the player has used
    /// (`input::profiles`).
    pub profiles: Mutex<[Vec<rc2k_core::input::profile::Profile>; 4]>,
    /// Whose profile each of the race's two device records reads through: the number of the
    /// player's record, set when the race takes the players' controls.
    pub race_players: [AtomicU32; 2],
    /// The profile each device record reads through in the race, taken when the race sets the
    /// device up (`input::set_up_device`).
    pub race_profiles: Mutex<[Option<rc2k_core::input::profile::Profile>; 2]>,
    /// A player's profile as it was when the controller's bindings page was entered, for its
    /// "cancel changes" item to put back.
    pub profile_undo: Mutex<Option<(u32, rc2k_core::input::profile::Profile)>>,
    /// The first device's reading when the controller page began learning a control.
    pub learn_rest: Mutex<Option<rc2k_core::input::profile::RawInput>>,
    /// Set by the input thread when it has listed controllers, for the game's thread to give
    /// the players theirs (`input::device_ids::give_chosen`).
    pub devices_listed: std::sync::atomic::AtomicBool,
    /// The keyboard and the mouse: only the game's thread reaches them (the input thread
    /// polls the devices, the keys and the mouse come from the window's events).
    main: GameThread<InputMain>,
}

/// The pad's presses at one moment (`Input::pad_marker`).
#[derive(Clone, Copy)]
pub struct PadMarker {
    presses: u32,
    starts: u32,
}

/// The keyboard and the mouse.
pub struct InputMain {
    /// The keyboard layout and the typed-key state.
    pub keyboard: Keyboard,
    /// The mouse position and buttons.
    pub mouse: Mouse,
}

impl Input {
    /// The pad's presses so far: a loop that ends on any key notes this at its start.
    pub fn pad_marker(&self) -> PadMarker {
        PadMarker {
            presses: self.pad_presses.load(Relaxed),
            starts: self.pad_starts.load(Relaxed),
        }
    }

    /// Whether A, B or Start (or a wheel's first two buttons) went down since `marker`.
    pub fn pad_pressed_since(&self, marker: PadMarker) -> bool {
        self.pad_presses.load(Relaxed) != marker.presses
    }

    /// Whether Start went down since `marker`.
    pub fn pad_start_since(&self, marker: PadMarker) -> bool {
        self.pad_starts.load(Relaxed) != marker.starts
    }

    /// A pad's A, B or Start (or a wheel's first two buttons) went down; `start`: it was Start.
    pub fn note_pad_press(&self, start: bool) {
        self.pad_presses.fetch_add(1, Relaxed);
        if start {
            self.pad_starts.fetch_add(1, Relaxed);
        }
    }

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
            pad_presses: AtomicU32::new(0),
            pad_starts: AtomicU32::new(0),
            learn_back_since: std::sync::atomic::AtomicU64::new(0),
            chosen_devices: Mutex::new([const { None }; 4]),
            profiles: Mutex::new([const { Vec::new() }; 4]),
            race_players: [AtomicU32::new(0), AtomicU32::new(1)],
            race_profiles: Mutex::new([const { None }; 2]),
            profile_undo: Mutex::new(None),
            learn_rest: Mutex::new(None),
            devices_listed: std::sync::atomic::AtomicBool::new(false),
            main: GameThread::new(InputMain {
                keyboard: Keyboard::new(),
                mouse: Mouse::new(),
            }),
        }
    }
}
