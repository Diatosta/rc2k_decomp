//! Module strings's owner (step 11, owned state): the text buffer, the keyboard tables, the
//! language and the lexicon in one struct, reached through `strings()` by the code that does
//! not take it as a parameter.

use crate::session::{GameThread, PlainBytes, PlainDword};
use crate::settings::read_driver_names::NAMES_SIZE;
use crate::strings::compile_lexicon::{EMPTY_MESSAGE_LEN, LexiconMemory, WORK_LEN};
use crate::strings::keyboard_tables::KeyboardTables;
use crate::strings::load_language::{LOCALE_INIT, LOCALE_LEN, PLAIN_LETTERS_INIT};
use crate::strings::text_buffer::TextBuffer;

/// The text buffer and the font blend mode.
pub struct Text {
    pub buffer: TextBuffer,
    /// `g_fontBlendMode` (0x4F3F74, the setting `textflashmode`, through `crate::moved::place`):
    /// 0 = no palette is made, 2 = eased. 3 in the image.
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
    /// The `.loc` file's 0x30 bytes (`g_localeFile`).
    pub locale: PlainBytes<LOCALE_LEN>,
    /// `g_plainLetters`.
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
    /// `aName000` (0x529DA5 in the image, 8 bytes): the key, made in place.
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

/// The strings' state.
pub struct Strings {
    pub text: Text,
    pub language: Language,
    pub lexicon: Lexicon,
    pub drivers: Drivers,
}

/// The one instance. Reached through `strings()`; `moved` registers its stores' places.
pub static STRINGS: GameThread<Strings> = GameThread::new(Strings {
    text: Text::new(),
    language: Language::new(),
    lexicon: Lexicon::new(),
    drivers: Drivers::new(),
});

/// The strings' state, for a caller that does not take it as a parameter yet.
pub fn strings() -> &'static Strings {
    STRINGS.get()
}
