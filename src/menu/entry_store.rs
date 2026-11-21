//! The key-entry state of the screens: whether a text is being typed, whether a key is being
//! redefined, and the sound the last key asks for.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// What the screens' text and key entry are doing. Plain values behind one `RefCell`: the
/// game's thread alone reads and writes them, each borrow lasting one statement.
pub struct EntryState(GameThread<RefCell<EntryValues>>);

/// The key-entry values.
#[derive(Clone, Copy)]
pub struct EntryValues {
    /// The menu's sound: 0x6F, or 0x14D after Enter (`menu::handle_keys`).
    pub menu_sound: u32,
    /// True while a text is being typed (`menu::text_entry`).
    pub typing: bool,
    /// True while a key is being redefined; also true = the highlight does not blink
    /// (`menu::update_blink`).
    pub redefining_key: bool,
    /// The place of the key being redefined (999 = none).
    pub redefined_key_place: u32,
}

impl EntryState {
    pub const fn new() -> Self {
        EntryState(GameThread::new(RefCell::new(EntryValues {
            menu_sound: 0x6F,
            typing: false,
            redefining_key: false,
            redefined_key_place: 0x3E7,
        })))
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> EntryValues {
        *self.0.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, EntryValues> {
        self.0.get().borrow_mut()
    }

    /// What the last key looked at was.
    pub fn sound(&self) -> rc2k_core::menu::keys::MenuSound {
        rc2k_core::menu::keys::MenuSound::from_code(self.read().menu_sound)
    }

    pub(super) fn set_sound(&self, sound: rc2k_core::menu::keys::MenuSound) {
        self.write().menu_sound = sound.code();
    }
}
