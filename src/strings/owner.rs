//! The strings' owned state: the text buffer, the keyboard tables, the language and the lexicon in
//! one struct, handed by reference to the code that uses them.

use crate::memory::session::{PlainBytes, PlainDword};
use crate::settings::read_driver_names::NAMES_SIZE;
use crate::strings::compile_lexicon::{EMPTY_MESSAGE_LEN, LexiconMemory, WORK_LEN};
use crate::strings::format_seal_code::SEAL_TEXT_LEN;
use crate::strings::load_language::{LOCALE_INIT, LOCALE_LEN, PLAIN_LETTERS_INIT};
use crate::strings::string_table::LEXICON_ROOM;
use crate::strings::text_buffer::TextBuffer;

/// The text buffer and the font blend mode.
pub struct Text {
    pub buffer: TextBuffer,
    /// The font blend mode (the setting `textflashmode`): 0 = no palette is made, 2 = eased. Starts
    /// at 3.
    pub blend_mode: PlainDword,
    /// 1 = a time is always written with its hours (0 at the start); the network's lobby sets it
    /// for a while.
    pub show_hours: PlainDword,
    /// The seal code's text (`format_seal_code`).
    pub seal_text: PlainBytes<SEAL_TEXT_LEN>,
}

impl Text {
    pub const fn new() -> Self {
        Text {
            buffer: TextBuffer::new(),
            blend_mode: PlainDword::new(3),
            show_hours: PlainDword::new(0),
            seal_text: PlainBytes::new([0; SEAL_TEXT_LEN]),
        }
    }
}

/// The language and keyboard tables.
pub struct Language {
    /// The `.loc` file's 0x30 bytes.
    pub locale: PlainBytes<LOCALE_LEN>,
    /// The plain-letters table.
    pub plain_letters: PlainBytes<256>,
    /// 0 = imperial (Fahrenheit, miles) and non-zero = metric (0 at the start). Set by the language
    /// (`load_language`) and player 1's choice (`menu::take_player1_units`, `menu::options_loop`).
    pub units: PlainDword,
    /// The built-in own texts of the language (`strings::own_texts`); none: English.
    pub own: std::sync::RwLock<Option<&'static rc2k_core::strings::own::File>>,
}

impl Language {
    pub const fn new() -> Self {
        Language {
            locale: PlainBytes::new(LOCALE_INIT),
            plain_letters: PlainBytes::new(PLAIN_LETTERS_INIT),
            units: PlainDword::new(0),
            own: std::sync::RwLock::new(None),
        }
    }
}

/// The lexicon compiler's memory.
pub struct Lexicon {
    /// The work variables.
    pub work: PlainBytes<WORK_LEN>,
    pub memory: LexiconMemory,
    /// The record all empty messages share.
    pub empty_message: PlainBytes<EMPTY_MESSAGE_LEN>,
    /// 1 or more (signed) to recompile on the numpad plus key: a `var\ral.cfg` setting, 0 unless set.
    pub recompile_level: PlainDword,
    /// The lexicon's room: where `compile_lexicon` puts the table.
    pub room: PlainBytes<LEXICON_ROOM>,
    /// The finished table as the core reads it: a copy of the room, made once its bytes are written
    /// (`string_table::publish`). Readers never see the room itself, so a text being looked up
    /// cannot meet a language being loaded.
    pub published: std::sync::RwLock<Option<&'static [u8]>>,
}

impl Lexicon {
    pub const fn new() -> Self {
        Lexicon {
            work: PlainBytes::new([0; WORK_LEN]),
            memory: LexiconMemory::new(),
            empty_message: PlainBytes::new([0; EMPTY_MESSAGE_LEN]),
            recompile_level: PlainDword::new(0),
            room: PlainBytes::new([0; LEXICON_ROOM]),
            published: std::sync::RwLock::new(None),
        }
    }
}

/// The driver names' work areas (`settings::read_driver_names`).
pub struct Drivers {
    /// The key, made in place (8 bytes).
    pub key: PlainBytes<8>,
    pub names: PlainBytes<NAMES_SIZE>,
}

impl Drivers {
    pub const fn new() -> Self {
        Drivers {
            key: PlainBytes::new(*b"name000\0"),
            names: PlainBytes::new([0; NAMES_SIZE]),
        }
    }
}

pub struct Strings {
    pub text: Text,
    pub language: Language,
    pub lexicon: Lexicon,
    pub drivers: Drivers,
}

impl Strings {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
            text: Text::new(),
            language: Language::new(),
            lexicon: Lexicon::new(),
            drivers: Drivers::new(),
        }
    }
}
