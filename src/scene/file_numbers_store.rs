//! The object and picture loaders' file numbers: the .ro3 / .lwo name numbers and paths, the
//! picture number and texture. The path strings between them stay unknown_* bytes, reached by
//! address.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ObjectFileNumbers {
    /// The dword before the file numbers; nothing reads or writes it (a gap).
    pub unknown_0: PlainDword,
    /// .ro3 name number, kept as the text's game address within `path_texts`
    /// (`scene::set_object_file_names`).
    pub object_ro3_number: PlainDword,
    /// .ro3 path, kept as the text's game address within `path_texts`
    /// (`scene::set_object_file_names`).
    pub object_ro3_path: PlainDword,
    /// .lwo path, kept as the text's game address within `path_texts`
    /// (`scene::set_object_file_names`).
    pub object_lwo_path: PlainDword,
    /// .lwo name number, kept as the text's game address within `path_texts`
    /// (`scene::set_object_file_names`).
    pub object_lwo_number: PlainDword,
    /// The path texts the stored text addresses above point into: the folder, map, info and
    /// car-icon file names (`texts`, reached by game address through `text_at`; the dwords are
    /// only the storage, 464 bytes of NUL-ended strings).
    pub path_texts: [PlainDword; 116],
    /// The picture number (`scene::load_texture_picture`).
    pub picture_number: PlainDword,
    /// The texture (`scene::load_texture_picture`).
    pub picture_texture: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl ObjectFileNumbers {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            path_texts: path_texts(&[
                (0, b"files\\s\\data\\titledem.pcx\0"),
                (26, b"files\\a\\obj????.ro3\0"),
                (46, b"files\\a\\obj????.lwo\0"),
                (66, b"files\\b\\obj????.ro3\0"),
                (86, b"files\\b\\obj????.lwo\0"),
                (106, b"files\\g\\frnt\\obj????.ro3\0"),
                (131, b"files\\g\\frnt\\obj????.lwo\0"),
                (156, b"files\\s\\extr\\obj????.ro3\0"),
                (181, b"files\\s\\extr\\obj????.lwo\0"),
                (206, b"files\\r\\errorxx\0"),
                (222, b"srcmaps\\mapxx.pcx\0"),
                (240, b"srcmaps\\keyxx.pcx\0"),
                (258, b"files\\a\\textinfo\0"),
                (275, b"files\\m\\markinfo\0"),
                (292, b"files\\b\\textinfo\0"),
                (309, b"files\\a\\objsinfo\0"),
                (326, b"files\\b\\carsinfo\0"),
                (343, b"files\\s\\extr\\extrinfo\0"),
                (365, b"files\\g\\frnt\\frntinfo\0"),
                (387, b"files\\b\\tx1822.pcx\0"),
                (406, b"files\\b\\tx1823.pcx\0"),
                (425, b"files\\b\\tx1824.pcx\0"),
                (444, b"files\\b\\tx1825.pcx\0"),
            ]),
            ..ZeroCells::ZERO
        }
    }
}

/// The path texts in `path_texts`, by game address (for `ObjectFileNumbers::text_at`).
pub mod texts {
    pub const FOLDER_0_RO3: usize = 0x60AD4E;
    pub const FOLDER_0_LWO: usize = 0x60AD62;
    pub const FOLDER_1_RO3: usize = 0x60AD76;
    pub const FOLDER_1_LWO: usize = 0x60AD8A;
    pub const FOLDER_2_RO3: usize = 0x60AD9E;
    pub const FOLDER_2_LWO: usize = 0x60ADB7;
    pub const FOLDER_3_RO3: usize = 0x60ADD0;
    pub const FOLDER_3_LWO: usize = 0x60ADE9;
    pub const ERROR_LIST_PATH: usize = 0x60AE02;
    pub const MAP_PATH: usize = 0x60AE12;
    pub const NUMBER_PLACE_A: usize = 0x60AE0F;
    pub const NUMBER_PLACE_B: usize = 0x60AE1D;
    pub const NUMBER_PLACE_C: usize = 0x60AE2F;
    pub const MAP_KEY: usize = 0x60AE24;
    pub const TEXTURE_INFO_A: usize = 0x60AE36;
    pub const MARK_INFO: usize = 0x60AE47;
    pub const TEXTURE_INFO_B: usize = 0x60AE58;
    pub const OBJECTS_INFO: usize = 0x60AE69;
    pub const CARS_INFO: usize = 0x60AE7A;
    pub const EXTRAS_INFO: usize = 0x60AE8B;
    pub const FRONT_INFO: usize = 0x60AEA1;
    pub const CAR_ICON_0: usize = 0x60AEB7;
    pub const CAR_ICON_1: usize = 0x60AECA;
    pub const CAR_ICON_2: usize = 0x60AEDD;
    pub const CAR_ICON_3: usize = 0x60AEF0;
}

/// A path text in `path_texts`: read up to its NUL and written to inside the 464 bytes the
/// texts occupy, its bounds.
pub struct PathText<'a> {
    numbers: &'a ObjectFileNumbers,
    /// The text's first byte, within `path_texts`.
    from: usize,
}

impl ObjectFileNumbers {
    /// The text at game address `ida`.
    pub fn text_at(&self, ida: usize) -> PathText<'_> {
        assert!((0x60AD34..0x60AF04).contains(&ida));
        PathText {
            numbers: self,
            from: ida - 0x60AD34,
        }
    }
}

impl crate::session::TextPlace for PathText<'_> {
    fn text(&self, _what: &str) -> Result<Vec<u8>, crate::errors::BadInput> {
        let table = &self.numbers.path_texts;
        let mut text = Vec::new();
        for n in self.from..TEXTS_LEN {
            match crate::session::byte_of(table, n) {
                Some(0) => return Ok(text),
                Some(c) => text.push(c),
                None => break,
            }
        }
        Err(crate::errors::BadInput("a text with no end in its buffer"))
    }

    fn write_text_at(
        &self,
        at: usize,
        bytes: &[u8],
        _what: &str,
    ) -> Result<(), crate::errors::BadInput> {
        let from = self.from.saturating_add(at);
        if from.saturating_add(bytes.len()) > TEXTS_LEN {
            return Err(crate::errors::BadInput("a write past the path texts"));
        }
        for (n, &byte) in (from..).zip(bytes) {
            let cell = &self.numbers.path_texts[n / 4];
            let mut word = cell.get().to_le_bytes();
            word[n % 4] = byte;
            cell.set(u32::from_le_bytes(word));
        }
        Ok(())
    }
}

/// The length of `path_texts` in bytes.
const TEXTS_LEN: usize = 4 * 116;

/// The path buffer whole: each text with its end byte at its byte offset, zero elsewhere.
const fn path_texts(texts: &[(usize, &[u8])]) -> [PlainDword; 116] {
    let mut bytes = [0u8; TEXTS_LEN];
    let mut t = 0;
    while t < texts.len() {
        let (at, text) = texts[t];
        let mut k = 0;
        while k < text.len() {
            bytes[at + k] = text[k];
            k += 1;
        }
        t += 1;
    }
    let mut dwords = [const { PlainDword::new(0) }; 116];
    let mut n = 0;
    while n < 116 {
        dwords[n] = PlainDword::new(u32::from_le_bytes([
            bytes[4 * n],
            bytes[4 * n + 1],
            bytes[4 * n + 2],
            bytes[4 * n + 3],
        ]));
        n += 1;
    }
    dwords
}
