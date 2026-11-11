//! What a key means on a menu (from `menu::handle_keys` 0x465FCC).
//! The keys are the game's scan codes as `input::take_key` gives them.

/// What a key means to the screen loop (`handle_keys`' answers 1, 2, 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pressed {
    /// Nothing for the caller to do.
    Nothing = 1,
    Escape = 2,
    Enter = 3,
}

/// What the last key looked at was, as the menus' sound variable keeps it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuSound {
    /// Any key looked at that was not Enter.
    None,
    /// Enter was pressed: the highlighted item's action runs.
    Enter,
}

impl MenuSound {
    const NONE_CODE: u32 = 0x6F;
    const ENTER_CODE: u32 = 0x14D;

    /// The value the game's variable holds.
    pub const fn code(self) -> u32 {
        match self {
            MenuSound::None => Self::NONE_CODE,
            MenuSound::Enter => Self::ENTER_CODE,
        }
    }

    pub const fn from_code(code: u32) -> MenuSound {
        if code == Self::ENTER_CODE {
            MenuSound::Enter
        } else {
            MenuSound::None
        }
    }
}

/// The menu's sound Enter plays.
pub const ENTER_CLICK: u32 = 2;

/// The keys.
pub const ESCAPE: u8 = 1;
pub const ENTER: u8 = 0x1C;
pub const IGNORED: u8 = 0x38;

/// The cheat bit that lets key 0x14 add time (bit 14 of `g_cheats`), and the time it adds
/// to the service's budget.
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
    /// Escape: `LoopFlags.leave_screen`, `DrawWork.escape_pressed` and `CaptionTables.left_by_key` 1; answer 2.
    Escape,
    /// Enter: the sound 0x14D, the key's time, `g_enterSkipsNameEntry` by the key code's second
    /// byte, the click; answer 3.
    Enter,
    /// The key's time, then one of the four highlight moves.
    Move(Move),
    /// The key's time, then a screenshot (key 0x52).
    Screenshot,
    /// The key's time, then `LoopFlags.toggle` turned over (key 0x20).
    Toggle,
    /// The key's time, then 60 added to the service's budget (key 0x14 with the cheat).
    AddTime,
    /// The key's time only.
    Other,
}

/// The keys are not looked at while an item is edited (`MenuState.service_ui.box_shown` 1) or with
/// `ControlsScreen.learning` 1 (`keys_off` asked only when not editing).
pub fn keys_off(editing: bool, keys_off: impl FnOnce() -> u32) -> bool {
    editing || keys_off() == 1
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

/// Enter's `g_enterSkipsNameEntry`: 1 when the key code's second byte is 4.
pub fn enter_skips_name_entry(key_code: u32) -> u32 {
    (key_code >> 8 & 0xFF == 4) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions() {
        assert!(keys_off(true, || unreachable!()));
        assert!(keys_off(false, || 1));
        assert!(!keys_off(false, || 0));
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
