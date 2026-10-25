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
use crate::net::lobby::{g, movsb};
use crate::race::state_store::RACE_STATE;
use rc2k_core::net::settings::{self, Copy};

const PLAYERS: usize = 0x75647C; // g_players
/// The Rust version: the registers it leaves (the entry's).
pub fn take_player_settings() -> Result<(), Refused> {
    let record = g(PLAYERS);
    if record == 0 {
        return Err(BadInput("no record for player 1").into());
    }
    // Converted: `rc2k_core::net::settings` (`copies`, `TAKEN_MARK`).
    unsafe { ((record + settings::TAKEN_MARK) as *mut u32).write_unaligned(1) };
    let to = RACE_STATE.settings.car_kind_current.addr() as u32;
    for copy in settings::copies() {
        match copy {
            Copy::Dword { to: at, from } => {
                let v = unsafe { ((record + from) as *const u32).read_unaligned() };
                unsafe { ((to + at) as *mut u32).write_unaligned(v) };
            }
            Copy::Bytes { to: at, from, len } => movsb(to + at, record + from, len),
        }
    }
    Ok(())
}
