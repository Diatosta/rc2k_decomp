//! `net::take_player_settings` (0x48755D): player 1's record (`g_players[0]`) copied into
//! the network game's settings (`dword_75ACE8` .. `byte_75AE87`): 91 dwords (the car,
//! the driving aids and the HUD's switches, the controls, the co-driver, the view; the
//! dword at `dword_75AD8C` twice, the second one kept), 12 bytes from +0x1C, 0x20 bytes from
//! +0x24C to `dword_75B06C`; and the record's +0xD0 made 1.
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! A null `g_players[0]` (the original reads through it) is refused. Caller:
//! `net::start_link`.

use crate::errors::{BadInput, Refused};
use crate::menu::players_store::PLAYER_TABLE;
use crate::net::lobby::movsb;
use crate::pointers::GamePtr;
use crate::race::state_store::RACE_STATE;
use rc2k_core::net::settings::{self, Copy};

/// The Rust version: the registers it leaves (the entry's).
pub fn take_player_settings() -> Result<(), Refused> {
    let record = PLAYER_TABLE.pointers[0].get();
    if record == 0 {
        return Err(BadInput("no record for player 1").into());
    }
    // Converted: `rc2k_core::net::settings` (`copies`, `TAKEN_MARK`).
    GamePtr::new(record + settings::TAKEN_MARK).write(1u32, "a player's record")?;
    let to = RACE_STATE.settings.car_kind_current.addr() as u32;
    for copy in settings::copies() {
        match copy {
            Copy::Dword { to: at, from } => {
                let v = GamePtr::new(record + from).read::<u32>("a player's record")?;
                GamePtr::new(to + at).write(v, "the race settings")?;
            }
            Copy::Bytes { to: at, from, len } => movsb(to + at, record + from, len)?,
        }
    }
    Ok(())
}
