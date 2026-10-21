//! The menus' one-line text entry (converted 2026-10-21 from its four editors:
//! `menu::edit_player_name` 0x467C63, `edit_savegame_name`, `edit_replay_name` and
//! `service::edit_setup_name` 0x46F1A9): what a pending key does. Each editor does its
//! own with it (the name brought back, the slot saved, the cursor written).

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
}
