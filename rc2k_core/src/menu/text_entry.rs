//! The menus' one-line text entry (from its four editors:
//! `menu::edit_player_name` 0x467C63, `edit_savegame_name`, `edit_replay_name` and
//! `service::edit_setup_name` 0x46F1A9): what a pending key does. Each editor does its
//! own with it (the name brought back, the slot saved, the cursor written). And the
//! entry's state and its helpers (from `menu::backup_text` 0x46D5CC,
//! `restore_text` 0x46D5E3, `clear_text` 0x467DC9, `add_text_length` 0x46F4CE,
//! `backspace_savegame_name` 0x46D5FD, `begin_player_name` 0x467C34, `begin_savegame_name`
//! 0x46D571 and `begin_replay_name` 0x46F715); the players' names fitted to their width
//! (`menu::fit_player_names` 0x411445).

use crate::Invalid;

/// The bytes the helpers copy and clear: a text field is 0x20 bytes with its NUL.
pub const TEXT_LEN: usize = 0x1F;

/// A text field's text, its NUL left out.
pub type Text = [u8; TEXT_LEN];

/// The entry's state: the length typed so far, 1 while a text is being
/// typed, the text as it was before (`Names.text_backup`, which Escape brings
/// back).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub length: i32,
    pub entering: bool,
    pub backup: Text,
}

impl Entry {
    /// `menu::backup_text`: the field's text kept.
    pub fn back_up(&mut self, field: &Text) {
        self.backup = *field;
    }

    /// `menu::restore_text`: the text kept, and the NUL after it.
    pub fn restored(&self) -> [u8; TEXT_LEN + 1] {
        let mut field = [0; TEXT_LEN + 1];
        field[..TEXT_LEN].copy_from_slice(&self.backup);
        field
    }

    /// `menu::clear_text`: the field emptied, nothing typed.
    pub fn clear(&mut self, field: &mut Text) {
        *field = [0; TEXT_LEN];
        self.length = 0;
    }

    /// `menu::add_text_length`: `len` more typed (its callers: a text just restored).
    pub fn add_length(&mut self, len: usize) {
        self.length = self.length.wrapping_add(len as i32);
    }

    /// The saved game and replay names' start: the text kept, the field cleared, an entry
    /// going on, nothing typed.
    pub fn begin(&mut self, field: &mut Text) {
        self.back_up(field);
        self.clear(field);
        self.entering = true;
        self.length = 0;
    }

    /// `menu::begin_player_name`: unless the Enter came with the second byte 4 (`with_4`), the name
    /// kept in `kept` (the players' own copy), cleared and an entry going on; either way
    /// nothing typed.
    pub fn begin_player_name(&mut self, with_4: bool, name: &mut Text, kept: &mut Text) {
        if !with_4 {
            *kept = *name;
            self.clear(name);
            self.entering = true;
        }
        self.length = 0;
    }

    /// The last character taken off a name of `room` bytes: the length one less, and the
    /// place of the NUL to write; none when nothing is typed. Refused: a length outside the
    /// name (not below `room`, or negative).
    pub fn backspace(&mut self, room: usize) -> Result<Option<usize>, Invalid> {
        if self.length == 0 {
            return Ok(None);
        }
        if !(0..room as i32).contains(&self.length) {
            return Err(Invalid("text length outside the name"));
        }
        self.length -= 1;
        Ok(Some(self.length as usize))
    }
}

/// The widest a player's name may be shown, in pixels (`menu::fit_player_names`).
pub const WIDEST_NAME: u32 = 0x15E;

/// `menu::fit_name` (0x411474, of `menu::fit_player_names` 0x411445): a name of `len`
/// characters cut, one character at a time, until it is no wider than `WIDEST_NAME`;
/// `measure(len)` ends the name at `len` (its NUL written) and measures it. The width it is
/// left with. Refused: an empty name still too wide.
pub fn fit_name<E: From<Invalid>>(
    len: usize,
    mut measure: impl FnMut(usize) -> Result<u32, E>,
) -> Result<u32, E> {
    let mut len = len;
    loop {
        let width = measure(len)?;
        if width <= WIDEST_NAME {
            return Ok(width);
        }
        if len == 0 {
            return Err(Invalid("an empty name that measures too wide").into());
        }
        len -= 1;
    }
}

/// The longest a text is looked through for its NUL.
pub const LONGEST: usize = 0x1000;

/// The length of a text (`byte` its bytes, asked in order up to its NUL). Refused: no NUL
/// in the first `LONGEST` bytes.
pub fn text_length(byte: impl Fn(usize) -> u8) -> Result<usize, Invalid> {
    (0..LONGEST)
        .find(|&i| byte(i) == 0)
        .ok_or(Invalid("text without a NUL"))
}

/// What a key asks of the entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// No key.
    None,
    /// 1: the name from before comes back, the entry ends.
    Escape,
    /// 0x1C: an empty name is replaced by the one from before; the entry ends.
    Enter,
    /// 0x6E: the name cleared.
    Clear,
    /// 0x0E: the last character taken off.
    Backspace,
    /// Any other: its character (`key_to_char`) typed.
    Other(u8),
}

pub fn key(k: u8) -> Key {
    match k {
        0 => Key::None,
        1 => Key::Escape,
        0x1C => Key::Enter,
        0x6E => Key::Clear,
        0x0E => Key::Backspace,
        k => Key::Other(k),
    }
}

/// Whether a typed character is appended: not 0, and the length not above the maximum
/// (signed).
pub fn appends(c: u8, len: i32, max: i32) -> bool {
    c != 0 && len <= max
}

/// The widest a name may measure, in pixels: one this wide or wider loses the character.
pub const WIDEST: i32 = 0xEC;

pub fn too_wide(width: i32) -> bool {
    width >= WIDEST
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry() {
        assert_eq!(
            (key(0), key(1), key(0x1C), key(0x6E), key(0x0E), key(0x10)),
            (
                Key::None,
                Key::Escape,
                Key::Enter,
                Key::Clear,
                Key::Backspace,
                Key::Other(0x10)
            )
        );
        assert!(appends(b'a', 5, 5) && !appends(b'a', 6, 5) && !appends(0, 0, 5));
        assert!(too_wide(0xEC) && !too_wide(0xEB));
    }

    #[test]
    fn state() {
        let mut e = Entry {
            length: 3,
            entering: false,
            backup: [0xBB; TEXT_LEN],
        };
        let mut field = [b'x'; TEXT_LEN];
        e.begin(&mut field);
        assert_eq!(
            (e.length, e.entering, e.backup, field),
            (0, true, [b'x'; TEXT_LEN], [0; TEXT_LEN])
        );
        assert_eq!(e.restored()[..TEXT_LEN], [b'x'; TEXT_LEN]);
        assert_eq!(e.restored()[TEXT_LEN], 0);
        e.add_length(5);
        assert_eq!(e.backspace(0x20), Ok(Some(4)));
        e.length = 0x20;
        assert!(e.backspace(0x20).is_err());
        e.length = 0;
        assert_eq!(e.backspace(0x20), Ok(None));
        let (mut name, mut kept) = ([b'n'; TEXT_LEN], [0; TEXT_LEN]);
        e = Entry {
            length: 7,
            entering: false,
            backup: [0; TEXT_LEN],
        };
        e.begin_player_name(true, &mut name, &mut kept);
        assert_eq!(
            (e.length, e.entering, name[0], kept[0]),
            (0, false, b'n', 0)
        );
        e.begin_player_name(false, &mut name, &mut kept);
        assert_eq!((e.entering, name[0], kept[0]), (true, 0, b'n'));
        let text = b"abc\0";
        assert_eq!(text_length(|i| text[i]), Ok(3));
        assert!(text_length(|_| 1).is_err());
        let mut cuts = Vec::new();
        let fit = fit_name(3, |len| -> Result<u32, Invalid> {
            cuts.push(len);
            Ok(0x100 + 0x40 * len as u32)
        });
        assert_eq!(fit, Ok(0x140));
        assert_eq!(cuts, [3, 2, 1]);
        assert!(fit_name(2, |_| Ok::<u32, Invalid>(0x200)).is_err());
    }
}
