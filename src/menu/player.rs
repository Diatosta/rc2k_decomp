//! A player, 0x28C bytes (`Player` in IDA): the four at `g_player1` .. `g_player4`,
//! pointed at by `g_players` (`off_75647C`): the player's settings (name, nationality,
//! views, co-driver, keys, controller) and the state of the player's car (which car,
//! damage, setup, wear). What is known of it; the rest are gaps. The service area's and
//! the saves' code still read it by offsets.

use crate::framework::{game_read, rva};

pub const PLAYERS_RVA: usize = rva(0x75647C); // g_players
pub const PLAYERS: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub gap_0: u32,
    /// The player's nationality (0 Great Britain, 1 Argentina, ..; `dword_758816` has
    /// their strings, the flag is sprite 0x1C + it).
    pub nationality: u32,
    /// The number of the player's car.
    pub car_number: u32,
    /// The view (`dword_7589BA`: bumper, bonnet, driver, interior, chase 1 .. 3).
    pub view: u32,
    /// The second view (`dword_7589DA`: bonnet, bumper, chase 1 .. 3; 5 is made 0).
    pub second_view: u32,
    pub field_14: u32,
    /// The player's name (a place in `g_playerNames`).
    pub name: u32,
    pub keys: [u8; 12],
    /// A number the player details screen shows.
    pub shown_number: u32,
    pub gap_2c: u32,
    pub field_30: u32,
    pub gap_34: [u8; 92],
    pub no_turbo: u32,
    pub gap_94: [u8; 56],
    pub damage: [u32; 16],
    pub tyre: u32,
    pub brake_balance: u32,
    pub ride_height: u32,
    pub suspension_stiffness: u32,
    pub steering_sensitivity: u32,
    pub gear_ratio: u32,
    pub name_copy: [u8; 32],
    pub field_144: u32,
    pub field_148: u32,
    pub field_14c: u32,
    pub field_150: u32,
    /// The co-driver's sex, from the player details page: 0 male, 1 female (strings
    /// 0x18A, 0x18B).
    pub codriver_sex: u32,
    pub gap_158: [u8; 36],
    pub wear_17c: [u32; 31],
    pub tyre_wear: [u32; 5],
    pub gap_20c: [u8; 64],
    pub field_24c: [u32; 8],
    pub gap_26c: [u8; 8],
    pub field_274: u32,
    pub flags_278: u32,
    pub gap_27c: u32,
    /// The controller's steering sensitivity, 0 .. 100 ("sensibilidade da direcção"; the
    /// player details page shows it as a marker at x + 2 * it).
    pub controller_sensitivity: u32,
    pub gap_284: [u8; 8],
}

const _: () = assert!(std::mem::size_of::<Player>() == 0x28C);

/// Player `index`'s address, or 0.
pub fn player_at(index: u32) -> u32 {
    if index >= PLAYERS {
        return 0;
    }
    unsafe { game_read(PLAYERS_RVA + index as usize * 4) }
}
