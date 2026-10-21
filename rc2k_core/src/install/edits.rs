//! The installer's edit items (kind 4) being edited (converted 2026-10-21 from
//! `install::start_edit` 0x4148D6 and `install::edit_key` 0x4148F7): an edit's record has
//! +0 its flags (bit 0 being edited), +1 the most bytes of its text, +2 its default text (a
//! string id), +4 its kind, +6 the widest its text may be.

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// A record's flag: being edited.
pub const EDITING: u8 = 1;

/// An edit begun: the record's flags with `EDITING`, and the text (`most` + 2 bytes) cut at
/// `most` (a NUL there); the backup kept for Escape is what the text then is.
pub fn start_edit(flags: u8, most: u8, text: &mut [u8]) -> u8 {
    text[most as usize] = 0;
    flags | EDITING
}

/// What a key does to an edit being edited.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKey {
    /// 1: the edit ended and the text put back from the backup.
    Escape,
    /// 0x1C: the edit ended; a blank text gets the record's default.
    Enter,
    /// 0x6E: the text cleared.
    Clear,
    /// 0xE: its last byte cut.
    Backspace,
    /// 0x66, 0x6B, 0x68, 0x69: nothing.
    Arrow,
    /// Any other: the key made a character (`input::key_to_char`).
    Other,
}

impl EditKey {
    /// The key's way, for a key that is not a character itself (ah bit 0 clear).
    pub fn of(key: u8) -> EditKey {
        match key {
            1 => EditKey::Escape,
            0x1C => EditKey::Enter,
            0x6E => EditKey::Clear,
            0xE => EditKey::Backspace,
            0x66 | 0x6B | 0x68 | 0x69 => EditKey::Arrow,
            _ => EditKey::Other,
        }
    }
}

/// Whether a text (to its NUL) is nothing but spaces and control bytes: Enter clears it
/// then. Where the look stopped is the byte past the NUL, or the first byte above a space.
pub fn blank(text: &(impl Bytes + ?Sized)) -> Result<(bool, usize), Invalid> {
    let mut n = 0;
    loop {
        let b = text
            .byte(n)
            .ok_or(Invalid("an edit's text without its NUL"))?;
        if b > 0x20 {
            return Ok((false, n));
        }
        n += 1;
        if b < 1 {
            return Ok((true, n));
        }
    }
}

/// Whether a character goes into the text: one of a space or more, with room for it (the
/// text's length not `most`; a longer one takes it too).
pub fn takes(ch: u8, length: usize, most: u8) -> bool {
    ch >= 0x20 && length != most as usize
}

/// Whether the text, measured `width` wide (signed), is wider than the record allows.
pub fn too_wide(width: u32, widest: u16) -> bool {
    (width as i32) > widest as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing() {
        let mut text = *b"abcdef\0\0";
        assert_eq!(start_edit(0x84, 4, &mut text), 0x85);
        assert_eq!(&text[..6], b"abcd\0f");
        assert_eq!(EditKey::of(1), EditKey::Escape);
        assert_eq!(EditKey::of(0x6B), EditKey::Arrow);
        assert_eq!(EditKey::of(0x20), EditKey::Other);
        assert_eq!(blank(b"  \t\0x"), Ok((true, 4)));
        assert_eq!(blank(b" a\0"), Ok((false, 1)));
        assert!(blank(b"  ").is_err());
        assert!(takes(b'a', 3, 4));
        assert!(!takes(b'a', 4, 4));
        assert!(!takes(0x1F, 0, 4));
        assert!(too_wide(0x31, 0x30));
        assert!(!too_wide(0x8000_0000, 0x30));
    }
}
