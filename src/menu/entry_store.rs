//! The key-entry state of the screens: whether a text is being typed, whether a key is being
//! redefined, and the sound the last key asks for.

use crate::session::{PlainDword, PlainFlag};

/// What the screens' text and key entry are doing.
#[derive(rc2k_derive::Cells)]
pub struct EntryState {
    /// The menu's sound: 0x6F, or 0x14D after Enter (`menu::handle_keys`).
    pub menu_sound: PlainDword,
    /// 1 while a text is being typed (`menu::text_entry`).
    pub typing: PlainFlag,
    /// 1 while a key is being redefined; also 1 = the highlight does not blink
    /// (`menu::update_blink`).
    pub redefining_key: PlainFlag,
    /// The place of the key being redefined (999 = none).
    pub redefined_key_place: PlainDword,
}

impl EntryState {
    pub const fn new() -> Self {
        EntryState {
            menu_sound: PlainDword::new(0x6F),
            typing: PlainFlag::new(false),
            redefining_key: PlainFlag::new(false),
            redefined_key_place: PlainDword::new(0x3E7),
        }
    }

    /// What the last key looked at was.
    pub fn sound(&self) -> rc2k_core::menu::keys::MenuSound {
        rc2k_core::menu::keys::MenuSound::from_code(self.menu_sound.get())
    }

    pub fn set_sound(&self, sound: rc2k_core::menu::keys::MenuSound) {
        self.menu_sound.set(sound.code());
    }
}
