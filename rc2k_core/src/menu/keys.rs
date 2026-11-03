//! What a key means on a menu (converted 2026-10-21 from `menu::handle_keys` 0x465FCC).
//! The keys are the game's scan codes as `input::take_key` gives them.

/// What a key means to the screen loop (`handle_keys`' eax 1, 2, 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pressed {
    /// Nothing for the caller to do.
    Nothing = 1,
    Escape = 2,
    Enter = 3,
}

/// `dword_7556A4` on any key looked at, and after Enter.
pub const NO_SOUND: u32 = 0x6F;
pub const ENTER_SOUND: u32 = 0x14D;

/// The menu's sound Enter plays.
pub const ENTER_CLICK: u32 = 2;

/// The keys.
pub const ESCAPE: u8 = 1;
pub const ENTER: u8 = 0x1C;
pub const IGNORED: u8 = 0x38;

/// The cheat bit that lets key 0x14 add time (bit 14 of `g_cheats`), and the time it adds
/// to the service's budget (`dword_75593C`).
pub const ADD_TIME_BIT: u32 = 14;
pub const ADDED_TIME: u32 = 60;

/// A highlight move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Up,
    Down,
    Right,
    Left,
}

/// What a key does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// No key: nothing.
    NoKey,
    /// Key 0x38: nothing (after the yes / no handling).
    Ignored,
    /// Escape: `dword_755561`, `dword_755A3C` and `dword_758806` 1; eax 2.
    Escape,
    /// Enter: the sound 0x14D, the key's time, `g_enterSkipsNameEntry` by ah, the click; eax 3.
    Enter,
    /// The key's time, then one of the four highlight moves.
    Move(Move),
    /// The key's time, then a screenshot (key 0x52).
    Screenshot,
    /// The key's time, then `dword_755575` turned over (key 0x20).
    Toggle,
    /// The key's time, then 60 added to the service's budget (key 0x14 with the cheat).
    AddTime,
    /// The key's time only.
    Other,
}

/// The keys are not looked at while an item is edited (`dword_75AA9C` 1) or with
/// `dword_7551F4` 1 (`keys_off` asked only when not editing).
pub fn keys_off(editing: u32, keys_off: impl FnOnce() -> u32) -> bool {
    editing == 1 || keys_off() == 1
}

/// The action of `key` (`cheats` asked only for key 0x14).
pub fn action(key: u8, cheats: impl FnOnce() -> u32) -> Action {
    match key {
        0 => Action::NoKey,
        IGNORED => Action::Ignored,
        ESCAPE => Action::Escape,
        ENTER => Action::Enter,
        0x66 => Action::Move(Move::Up),
        0x6B => Action::Move(Move::Down),
        0x69 => Action::Move(Move::Right),
        0x68 => Action::Move(Move::Left),
        0x52 => Action::Screenshot,
        0x20 => Action::Toggle,
        0x14 if cheats() >> ADD_TIME_BIT & 1 != 0 => Action::AddTime,
        _ => Action::Other,
    }
}

/// Enter's `g_enterSkipsNameEntry`: 1 when ah (what the caller had there) is 4.
pub fn enter_skips_name_entry(eax: u32) -> u32 {
    (eax >> 8 & 0xFF == 4) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions() {
        assert!(keys_off(1, || unreachable!()));
        assert!(keys_off(0, || 1));
        assert!(!keys_off(2, || 0));
        assert_eq!(action(0, || unreachable!()), Action::NoKey);
        assert_eq!(action(0x38, || unreachable!()), Action::Ignored);
        assert_eq!(action(1, || unreachable!()), Action::Escape);
        assert_eq!(action(0x1C, || unreachable!()), Action::Enter);
        assert_eq!(action(0x68, || unreachable!()), Action::Move(Move::Left));
        assert_eq!(action(0x14, || 1 << 14), Action::AddTime);
        assert_eq!(action(0x14, || !(1 << 14)), Action::Other);
        assert_eq!(action(0x30, || unreachable!()), Action::Other);
        assert_eq!(enter_skips_name_entry(0x0000_041C), 1);
        assert_eq!(enter_skips_name_entry(0x0004_001C), 0);
    }
}
