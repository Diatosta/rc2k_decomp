//! What the game keeps between runs, loaded, and the names that are empty given defaults
//! (converted 2026-10-21 from `system::load_game_data` 0x411239: its choices and the
//! names it makes; the loading and the Win32 names stay with the caller).

use crate::strings::format::{Pad, decimal};

/// `g_allSecretCars` (the setting `allsecretcars`): 0 removes the secret cars, 1 turns on
/// the cheats that give them all (read again after the removal).
pub fn removes_secret_cars(setting: u32) -> bool {
    setting == 0
}

pub fn all_secret_cars(setting: u32) -> bool {
    setting == 1
}

/// The cheats' bits that give every car.
pub const ALL_CARS_CHEATS: u32 = 0x1FBC;

/// The players' default names (strings), the first used only when the user's name gives
/// nothing.
pub const PLAYER_NAME_IDS: [u32; 4] = [0x18C, 0x18D, 0x18E, 0x18F];
/// The machine's default name.
pub const MACHINE_NAME_ID: u32 = 0x14D;
/// The room for the user's name.
pub const USER_NAME_ROOM: u32 = 0x1F;
/// The driver's name drawn from these strings: the first, and how many.
pub const DRIVER_NAME_IDS: (u32, u32) = (0x5A5, 0xB4);
/// A drawn driver's name may not start so (a developer's name: drawn again).
pub const TAKEN_PREFIX: [u8; 4] = *b"andr";
/// The tyre change's best time given with a new driver's name, in hundredths (210.00 s).
pub const TYRE_CHANGE_RECORD: u32 = 0x5208;
/// The slots of saved games, replays and car setups, and their default names' strings.
pub const SLOTS: u32 = 100;
pub const SAVEGAME_NAME_ID: u32 = 0x4E0;
pub const REPLAY_NAME_ID: u32 = 0x4E2;
pub const SETUP_NAME_ID: u32 = 0x4E1;

/// A default name: the string's bytes, what follows them, the NUL; None when it does not
/// fit `room`.
pub fn default_name(string: &[u8], after: &[u8], room: usize) -> Option<Vec<u8>> {
    let mut name = string.to_vec();
    name.extend_from_slice(after);
    name.push(0);
    (name.len() <= room).then_some(name)
}

/// A slot's default name: the string and the slot's number in two digits.
pub fn slot_name(string: &[u8], slot: u32, room: usize) -> Option<Vec<u8>> {
    default_name(string, &decimal(slot, Pad::from_flags(0x12)), room)
}

/// Whether a drawn driver's name (its first four bytes where it was written) must be
/// drawn again.
pub fn drawn_again(first: [u8; 4]) -> bool {
    first == TAKEN_PREFIX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert!(removes_secret_cars(0) && !removes_secret_cars(1));
        assert!(all_secret_cars(1) && !all_secret_cars(2));
        assert_eq!(
            default_name(b"Player", b"", 0x20),
            Some(b"Player\0".to_vec())
        );
        assert_eq!(default_name(b"Player", b"", 6), None);
        assert_eq!(slot_name(b"Save ", 7, 0x20), Some(b"Save 07\0".to_vec()));
        assert!(drawn_again(*b"andr") && !drawn_again(*b"Andr"));
    }
}
