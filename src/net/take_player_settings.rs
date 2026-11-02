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

use crate::errors::Refused;
use crate::menu::players_store::PLAYER_TABLE;
use crate::pointers::GamePtr;
use crate::race::state_store::RACE_STATE;
use rc2k_core::net::settings::{self, Copy};

/// The Rust version: the registers it leaves (the entry's).
pub fn take_player_settings() -> Result<(), Refused> {
    let player = &PLAYER_TABLE.players[0];
    // Converted: `rc2k_core::net::settings` (`copies`, `TAKEN_MARK`).
    player.update(|p| p.store(&[(settings::TAKEN_MARK as usize, 1)]));
    let to = RACE_STATE.settings.car_kind_current.addr();
    for copy in settings::copies() {
        match copy {
            Copy::Dword { to: at, from } => {
                let v = player.get().word(from as usize);
                GamePtr::new(to + at).write(v, "the race settings")?;
            }
            Copy::Bytes { to: at, from, len } => {
                let bytes = player.get().to_le_bytes();
                GamePtr::new(to + at).write_bytes(
                    &bytes[from as usize..(from + len) as usize],
                    "the race settings",
                )?;
            }
        }
    }
    Ok(())
}
