//! The on-screen keyboard a text field shows when it was entered from a controller: a grid of
//! keys moved with the cross or stick, and what pressing a key does.

/// Keys in each of the four character rows.
pub const COLUMNS: usize = 10;
/// The rows: four of characters and one of the command keys.
pub const ROWS: usize = 5;

const LOWER: [&[u8; COLUMNS]; 4] = [b"1234567890", b"qwertyuiop", b"asdfghjkl-", b"zxcvbnm._@"];
const UPPER: [&[u8; COLUMNS]; 4] = [b"1234567890", b"QWERTYUIOP", b"ASDFGHJKL-", b"ZXCVBNM._@"];

/// The command keys' row, left to right.
pub const COMMANDS: [Key; 4] = [Key::Shift, Key::Space, Key::Backspace, Key::Done];

/// What a key of the grid is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// A character, as its Windows-1252 byte.
    Char(u8),
    Shift,
    Space,
    Backspace,
    Done,
}

/// What pressing the key asks of the field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Press {
    Type(u8),
    Backspace,
    Done,
    /// The keyboard itself changed (case).
    Nothing,
}

/// The keyboard's cursor and case.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Osk {
    pub row: usize,
    pub col: usize,
    pub shifted: bool,
}

impl Osk {
    /// The number of keys in a row.
    pub fn width(row: usize) -> usize {
        if row < 4 { COLUMNS } else { COMMANDS.len() }
    }

    /// The key at a place of the grid.
    pub fn key_at(&self, row: usize, col: usize) -> Key {
        if row < 4 {
            let table = if self.shifted { UPPER } else { LOWER };
            Key::Char(table[row][col % COLUMNS])
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
        }
    }
}

/// The text a key is drawn with.
pub fn label(key: Key, shifted: bool) -> &'static [u8] {
    match key {
        Key::Char(_) => b"",
        Key::Shift if shifted => b"abc",
        Key::Shift => b"ABC",
        Key::Space => b"space",
        Key::Backspace => b"del",
        Key::Done => b"ok",
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
        assert_eq!((k.row, k.col), (0, 1));
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
        k.col = 1;
        assert_eq!(k.press(), Press::Type(b' '));
        k.col = 2;
        assert_eq!(k.press(), Press::Backspace);
        k.col = 3;
        assert_eq!(k.press(), Press::Done);
    }
}
