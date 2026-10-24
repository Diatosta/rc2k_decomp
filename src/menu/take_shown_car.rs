//! `menu::take_shown_car` (0x47B29D): the car shown on the car choice given to the current
//! player.
//!
//! In:  esi = the car list (pointers to the cars' entries)
//! Out: every register preserved
//!
//! The current player (`dword_75598C`) gets the entry `dword_755990` of the list: its +0 as
//! `car_number` (+8), its +8 as `no_turbo` (+0x90). Callers: `sub_482382` (after
//! `menu::select_car_loop`, esi the list it was given) and seven more. Refuses a current player
//! past the four (the original reads past `g_players`).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, rva};
use crate::menu::player::{PLAYERS, Player, player_at};

pub const CURRENT_PLAYER_RVA: usize = rva(0x75598C); // dword_75598C
pub const SHOWN_CAR_RVA: usize = rva(0x755990); // dword_755990

const _: () = assert!(std::mem::offset_of!(Player, car_number) == 8);
const _: () = assert!(std::mem::offset_of!(Player, no_turbo) == 0x90);

pub unsafe fn take_shown_car(list: u32) -> Result<(), Refused> {
    let current: u32 = unsafe { game_read(CURRENT_PLAYER_RVA) };
    if current >= PLAYERS {
        return Err(BadInput("a current player past the four").into());
    }
    let shown: u32 = unsafe { game_read(SHOWN_CAR_RVA) };
    // Converted: `rc2k_core::menu::player::Player::take_car`.
    unsafe {
        let entry = ((list.wrapping_add(shown.wrapping_mul(4))) as *const u32).read_unaligned();
        let number = (entry as *const u32).read_unaligned();
        let no_turbo = ((entry + 8) as *const u32).read_unaligned();
        let at = player_at(current) as *mut Player;
        let mut p = at.read_unaligned();
        p.take_car(number, no_turbo);
        at.write_unaligned(p);
    }
    Ok(())
}
