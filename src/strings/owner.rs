//! The strings' owned state: the text buffer, the keyboard tables, the language and the lexicon in
//! one struct, handed by reference to the code that uses them.

use crate::session::{PlainBytes, PlainDword};
use crate::settings::read_driver_names::NAMES_SIZE;
use crate::strings::compile_lexicon::{EMPTY_MESSAGE_LEN, LexiconMemory, WORK_LEN};
use crate::strings::keyboard_tables::KeyboardTables;
use crate::strings::load_language::{LOCALE_INIT, LOCALE_LEN, PLAIN_LETTERS_INIT};
use crate::strings::text_buffer::TextBuffer;

/// The text buffer and the font blend mode.
pub struct Text {
    pub buffer: TextBuffer,
    /// The font blend mode (the setting `textflashmode`): 0 = no palette is made, 2 = eased. Starts
    /// at 3.
    pub blend_mode: PlainDword,
}

impl Text {
    pub const fn new() -> Self {
        Text {
            buffer: TextBuffer::new(),
            blend_mode: PlainDword::new(3),
        }
    }
}

/// The language and keyboard tables.
pub struct Language {
    /// The `.loc` file's 0x30 bytes.
    pub locale: PlainBytes<LOCALE_LEN>,
    /// The plain-letters table.
    pub plain_letters: PlainBytes<256>,
    pub keyboard: KeyboardTables,
}

impl Language {
    pub const fn new() -> Self {
        Language {
            locale: PlainBytes::new(LOCALE_INIT),
            plain_letters: PlainBytes::new(PLAIN_LETTERS_INIT),
            keyboard: KeyboardTables::new(),
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
}

impl Lexicon {
    pub const fn new() -> Self {
        Lexicon {
            work: PlainBytes::new([0; WORK_LEN]),
            memory: LexiconMemory::new(),
            empty_message: PlainBytes::new([0; EMPTY_MESSAGE_LEN]),
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
