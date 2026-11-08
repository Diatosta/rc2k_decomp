//! The language's letter case: the lower-case table the language file sets up, and what the
//! game does with it (converted 2026-10-20 from `strings::same_text`, `copy_printable`).

use std::cmp::Ordering;

/// For each byte, its lower-case form in the language's character set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaseTable {
    pub lower: [u8; 256],
}

impl CaseTable {
    /// ASCII's letters only (the table before the language's own pairs are added).
    pub fn ascii() -> Self {
        CaseTable {
            lower: std::array::from_fn(|c| (c as u8).to_ascii_lowercase()),
        }
    }

    /// The language's table (RAL.EXE `strings::make_case_tables`): ASCII's letters, then `pairs`
    /// (the language's string 2) as "lower, upper" pairs. An odd length is refused (the original
    /// reads on past the string).
    pub fn for_language(pairs: &[u8]) -> Result<Self, crate::Invalid> {
        if !pairs.len().is_multiple_of(2) {
            return Err(crate::Invalid("the lower case string has an odd length"));
        }
        let mut table = CaseTable::ascii();
        for pair in pairs.chunks(2) {
            table.lower[pair[1] as usize] = pair[0];
        }
        Ok(table)
    }

    /// This table with accented letters made plain: in `accents` (the language's string 3,
    /// "a\xE0\xE1..e\xE8.."), each byte of 0x80 or more becomes the plain letter before it
    /// wherever it is a lower-case form.
    pub fn plain_letters(&self, accents: &[u8]) -> Self {
        let mut plain = *self;
        let mut letter = 0u8;
        for &c in accents {
            if c < 0x80 {
                letter = c;
            } else {
                for entry in plain.lower.iter_mut().filter(|entry| **entry == c) {
                    *entry = letter;
                }
            }
        }
        plain
    }

    pub fn lower(&self, c: u8) -> u8 {
        self.lower[c as usize]
    }

    /// Two texts compared without regard to case, character by character up to the first
    /// difference or the end (a text's end counts as a NUL).
    pub fn compare(&self, a: &[u8], b: &[u8]) -> Ordering {
        for i in 0.. {
            let x = self.lower(a.get(i).copied().unwrap_or(0));
            let y = self.lower(b.get(i).copied().unwrap_or(0));
            if x != y {
                return x.cmp(&y);
            }
            if x == 0 {
                break;
            }
        }
        Ordering::Equal
    }

    /// A name as it can be shown: at most `room - 1` characters of `text` (up to a NUL),
    /// leaving out those whose low seven bits are under 0x20, in lower case unless
    /// `keep_case`; then the NUL. `None` for a room of 0 (the original looks at 2^32 - 1).
    pub fn printable(&self, text: &[u8], room: usize, keep_case: bool) -> Option<Vec<u8>> {
        let looked_at = room.checked_sub(1)?;
        let mut out: Vec<u8> = text
            .iter()
            .take(looked_at)
            .take_while(|&&c| c != 0)
            .filter(|&&c| c & 0x7F >= 0x20)
            .map(|&c| if keep_case { c } else { self.lower(c) })
            .collect();
        out.push(0);
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> CaseTable {
        let mut t = CaseTable::ascii();
        t.lower[0xC9] = 0xE9; // É -> é
        t
    }

    #[test]
    fn compares() {
        let t = table();
        assert_eq!(t.compare(b"C:\\Games", b"c:\\games"), Ordering::Equal);
        assert_eq!(t.compare(b"\xC9t\xE9", b"\xE9T\xC9"), Ordering::Equal); // É is é
        assert_eq!(t.compare(b"abc", b"ab"), Ordering::Greater);
        assert_eq!(t.compare(b"", b""), Ordering::Equal);
        assert_eq!(t.compare(b"a", b"B"), Ordering::Less);
    }

    #[test]
    fn language_tables() {
        let t = CaseTable::for_language(b"\xE9\xC9\xE8\xC8").unwrap();
        assert_eq!(t.lower(0xC9), 0xE9);
        assert_eq!(t.lower(b'Q'), b'q');
        assert!(CaseTable::for_language(b"\xE9").is_err());
        let plain = t.plain_letters(b"e\xE9\xE8a\xE0");
        assert_eq!(plain.lower(0xC9), b'e');
        assert_eq!(plain.lower(0xE8), b'e');
        assert_eq!(plain.lower(0xE0), b'a');
        assert_eq!(plain.lower(b'E'), b'e');
    }

    #[test]
    fn printables() {
        let t = table();
        assert_eq!(t.printable(b"Ab\x01C\x81d", 10, false).unwrap(), b"abcd\0");
        assert_eq!(t.printable(b"AbC", 10, true).unwrap(), b"AbC\0");
        assert_eq!(t.printable(b"ABCDEF", 3, false).unwrap(), b"ab\0");
        assert_eq!(t.printable(b"AB\0CD", 10, false).unwrap(), b"ab\0");
        assert_eq!(t.printable(b"AB", 1, false).unwrap(), b"\0");
        assert_eq!(t.printable(b"AB", 0, false), None);
    }
}
