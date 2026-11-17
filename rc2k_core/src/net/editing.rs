//! The network screens' text being edited: one item's text at a time (the item noted, -1 for
//! none), 64 bytes kept for Escape, characters added while the text is narrower than allowed.
//! Like `crate::install::edits`, with the network's own keys and room.

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// The item edited when none is.
pub const NOT_EDITING: u32 = u32::MAX;
/// A text's bytes (cleared, and kept for Escape).
pub const TEXT_LEN: usize = 0x40;
/// The most characters a text takes.
pub const MOST_CHARS: usize = 0x3F;
/// The widest an edited text may be on the network screens.
pub const WIDTH: u32 = 0x30C;

/// What a key does to the text being edited.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKey {
    /// The kept text put back and the editing ended.
    Escape,
    /// The editing ended.
    Enter,
    /// The text's 64 bytes cleared.
    Clear,
    /// Its last character dropped.
    Backspace,
    /// Any other key: it makes a character (`input::key_to_char`).
    Other,
}

impl EditKey {
    pub fn of(key: u8) -> EditKey {
        match key {
            1 => EditKey::Escape,
            0x1C => EditKey::Enter,
            0x6E => EditKey::Clear,
            0x0E => EditKey::Backspace,
            _ => EditKey::Other,
        }
    }
}

/// The byte Backspace clears: the last before the text's NUL; `None` for an empty text.
/// Refused: a text with no end.
pub fn last_char(text: &(impl Bytes + ?Sized)) -> Result<Option<usize>, Invalid> {
    let first = text.byte(0).ok_or(Invalid("an edited text with no end"))?;
    if first == 0 {
        return Ok(None);
    }
    let mut n = 1;
    loop {
        match text.byte(n) {
            None => return Err(Invalid("an edited text with no end")),
            Some(0) => return Ok(Some(n - 1)),
            Some(_) => n += 1,
        }
    }
}

/// Whether a character typed goes into the text: 0x20 or more.
pub fn takes(ch: u8) -> bool {
    ch >= 0x20
}

/// Where a character goes: the text's NUL among its first `MOST_CHARS` bytes (`None`: the
/// text is full).
pub fn end_for_char(text: &(impl Bytes + ?Sized)) -> Result<Option<usize>, Invalid> {
    for n in 0..MOST_CHARS {
        match text.byte(n) {
            None => return Err(Invalid("an edited text shorter than its room")),
            Some(0) => return Ok(Some(n)),
            Some(_) => {}
        }
    }
    Ok(None)
}

/// Whether the text, measured `width` wide, is wider than `widest` (signed): the character
/// is taken off again.
pub fn too_wide(width: u32, widest: u32) -> bool {
    (width as i32) > (widest as i32)
}

/// The settings screen's items Enter starts editing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    /// The player's name.
    Name,
    /// The most players, edited as a number's text.
    MostPlayers,
    /// The host's address.
    HostAddress,
    /// The port, edited as a number's text.
    Port,
    /// Any other item: nothing is edited.
    None,
}

impl Setting {
    pub fn of(item: u32) -> Setting {
        match item {
            1 => Setting::Name,
            2 => Setting::MostPlayers,
            3 => Setting::HostAddress,
            4 => Setting::Port,
            _ => Setting::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys() {
        assert_eq!(EditKey::of(1), EditKey::Escape);
        assert_eq!(EditKey::of(0x1C), EditKey::Enter);
        assert_eq!(EditKey::of(0x6E), EditKey::Clear);
        assert_eq!(EditKey::of(0x0E), EditKey::Backspace);
        assert_eq!(EditKey::of(0x1E), EditKey::Other);
        assert_eq!(last_char(b"abc\0"), Ok(Some(2)));
        assert_eq!(last_char(b"\0x"), Ok(None));
        assert!(last_char(b"ab").is_err());
        assert!(takes(0x20) && !takes(0x1F));
        assert_eq!(end_for_char(b"ab\0"), Ok(Some(2)));
        assert_eq!(end_for_char(&[b'x'; 0x40]), Ok(None));
        let mut full = [b'x'; 0x40];
        full[0x3E] = 0;
        assert_eq!(end_for_char(&full), Ok(Some(0x3E)));
        assert!(end_for_char(b"ab").is_err());
        assert!(too_wide(0x30D, WIDTH));
        assert!(!too_wide(0x30C, WIDTH));
        assert!(!too_wide(0x8000_0000, WIDTH));
    }

    #[test]
    fn settings() {
        assert_eq!(Setting::of(1), Setting::Name);
        assert_eq!(Setting::of(2), Setting::MostPlayers);
        assert_eq!(Setting::of(3), Setting::HostAddress);
        assert_eq!(Setting::of(0), Setting::None);
        assert_eq!(Setting::of(4), Setting::Port);
        assert_eq!(Setting::of(5), Setting::None);
    }
}
