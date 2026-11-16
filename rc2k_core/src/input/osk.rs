//! The on-screen keyboard a text field shows when it was entered from a controller: a grid of
//! keys moved with the cross or stick, and what pressing a key does. Three pages of characters
//! (letters, symbols, accented letters), each in lower or upper case where that means anything.

/// Keys in each of the four character rows.
pub const COLUMNS: usize = 10;
/// The rows: four of characters and one of the command keys.
pub const ROWS: usize = 5;

const LETTERS: [&[u8; COLUMNS]; 4] = [b"1234567890", b"qwertyuiop", b"asdfghjkl-", b"zxcvbnm._@"];
const SYMBOLS: [&[u8; COLUMNS]; 4] = [b"1234567890", b"!?*#$%&()/", b"+=:;,'\"<>\\", b"-_.@[]{}~^"];
/// Windows-1252: the accented letters (lower case; `upper` gives the capitals) and a few signs.
const ACCENTS: [&[u8; COLUMNS]; 4] = [
    b"\xE1\xE0\xE2\xE3\xE4\xE5\xE6\xE7\xF1\xDF",
    b"\xE9\xE8\xEA\xEB\xED\xEC\xEE\xEF\xFD\xFF",
    b"\xF3\xF2\xF4\xF5\xF6\xF8\xFA\xF9\xFB\xFC",
    b"\xA1\xBF\xAB\xBB\xA9\xA3\xB0\xBA\xAA\xA7",
];

/// The command keys' row, left to right.
pub const COMMANDS: [Key; 6] = [
    Key::Shift,
    Key::Symbols,
    Key::Accents,
    Key::Space,
    Key::Backspace,
    Key::Done,
];

/// What a key of the grid is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// A character, as its Windows-1252 byte.
    Char(u8),
    Shift,
    /// The symbols page, or back to the letters from it.
    Symbols,
    /// The accented letters' page, or back to the letters from it.
    Accents,
    Space,
    Backspace,
    Done,
}

/// The page of characters shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Letters,
    Symbols,
    Accents,
}

/// What pressing the key asks of the field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Press {
    Type(u8),
    Backspace,
    Done,
    /// The keyboard itself changed (case or page).
    Nothing,
}

/// The keyboard's cursor, case and page.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Osk {
    pub row: usize,
    pub col: usize,
    pub shifted: bool,
    pub page: Page,
}

/// The capital of a Windows-1252 letter (ASCII and the accented ones); other bytes as they are.
pub fn upper(c: u8) -> u8 {
    match c {
        b'a'..=b'z' => c - 0x20,
        // à .. þ but ÷; ß and ÿ have no capital of their own in the code page's upper half.
        0xE0..=0xFE if c != 0xF7 => c - 0x20,
        _ => c,
    }
}

impl Osk {
    /// The number of keys in a row.
    pub fn width(row: usize) -> usize {
        if row < 4 { COLUMNS } else { COMMANDS.len() }
    }

    /// The key at a place of the grid.
    pub fn key_at(&self, row: usize, col: usize) -> Key {
        if row < 4 {
            let table = match self.page {
                Page::Letters => LETTERS,
                Page::Symbols => SYMBOLS,
                Page::Accents => ACCENTS,
            };
            let c = table[row][col % COLUMNS];
            let cased = self.shifted && self.page != Page::Symbols;
            Key::Char(if cased { upper(c) } else { c })
        } else {
            COMMANDS[col % COMMANDS.len()]
        }
    }

    /// The key under the cursor.
    pub fn key(&self) -> Key {
        self.key_at(self.row, self.col)
    }

    /// The cursor moved one key (wrapping at the edges); going between the command row and
    /// the rows above it keeps to the same part of the width.
    pub fn step(&mut self, dx: i32, dy: i32) {
        if dx != 0 {
            let width = Self::width(self.row) as i32;
            self.col = (self.col as i32 + dx).rem_euclid(width) as usize;
        }
        if dy != 0 {
            let to = (self.row as i32 + dy).rem_euclid(ROWS as i32) as usize;
            let (from_width, to_width) = (Self::width(self.row), Self::width(to));
            // The middle of the key's span, scaled to the other row.
            self.col = ((self.col * 2 + 1) * to_width / (from_width * 2)).min(to_width - 1);
            self.row = to;
        }
    }

    fn toggle_page(&mut self, page: Page) {
        self.page = if self.page == page {
            Page::Letters
        } else {
            page
        };
    }

    /// The key under the cursor pressed.
    pub fn press(&mut self) -> Press {
        match self.key() {
            Key::Char(c) => Press::Type(c),
            Key::Space => Press::Type(b' '),
            Key::Backspace => Press::Backspace,
            Key::Done => Press::Done,
            Key::Shift => {
                self.shifted = !self.shifted;
                Press::Nothing
            }
            Key::Symbols => {
                self.toggle_page(Page::Symbols);
                Press::Nothing
            }
            Key::Accents => {
                self.toggle_page(Page::Accents);
                Press::Nothing
            }
        }
    }

    /// The text a command key is drawn with (Windows-1252); none for a character key.
    pub fn label(&self, key: Key) -> &'static [u8] {
        match key {
            Key::Char(_) => b"",
            Key::Shift if self.shifted => b"abc",
            Key::Shift => b"ABC",
            Key::Symbols if self.page == Page::Symbols => b"abc",
            Key::Symbols => b"!?#",
            Key::Accents if self.page == Page::Accents => b"abc",
            Key::Accents => b"\xE1\xE9\xE7",
            Key::Space => b"space",
            Key::Backspace => b"del",
            Key::Done => b"ok",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_and_wraps() {
        let mut k = Osk::default();
        assert_eq!(k.key(), Key::Char(b'1'));
        k.step(-1, 0);
        assert_eq!(k.key(), Key::Char(b'0'));
        k.step(0, -1);
        assert_eq!(k.row, 4);
        assert_eq!(k.key(), Key::Done);
        k.step(1, 0);
        assert_eq!(k.key(), Key::Shift);
        k.step(0, 1);
        assert_eq!((k.row, k.col), (0, 0));
    }

    #[test]
    fn presses() {
        let mut k = Osk::default();
        k.step(0, 1);
        assert_eq!(k.press(), Press::Type(b'q'));
        k.row = 4;
        k.col = 0;
        assert_eq!(k.press(), Press::Nothing);
        k.row = 1;
        assert_eq!(k.press(), Press::Type(b'Q'));
        k.row = 4;
        k.col = 3;
        assert_eq!(k.press(), Press::Type(b' '));
        k.col = 4;
        assert_eq!(k.press(), Press::Backspace);
        k.col = 5;
        assert_eq!(k.press(), Press::Done);
    }

    #[test]
    fn pages() {
        let mut k = Osk {
            row: 4,
            col: 1,
            ..Osk::default()
        };
        assert_eq!(k.press(), Press::Nothing);
        assert_eq!(k.page, Page::Symbols);
        assert_eq!(k.key_at(1, 0), Key::Char(b'!'));
        // capitals leave the symbols as they are
        k.shifted = true;
        assert_eq!(k.key_at(1, 2), Key::Char(b'*'));
        k.col = 2;
        k.press();
        assert_eq!(k.page, Page::Accents);
        assert_eq!(k.key_at(0, 7), Key::Char(0xC7)); // Ç
        assert_eq!(k.key_at(0, 9), Key::Char(0xDF)); // ß stays
        k.press();
        assert_eq!(k.page, Page::Letters);
    }

    #[test]
    fn capitals() {
        assert_eq!(upper(b'a'), b'A');
        assert_eq!(upper(0xE3), 0xC3);
        assert_eq!(upper(0xF7), 0xF7);
        assert_eq!(upper(b'1'), b'1');
    }
}
