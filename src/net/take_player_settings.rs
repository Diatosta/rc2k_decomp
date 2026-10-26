//! `net::take_player_settings` (0x48755D): player 1's record (`g_players[0]`) copied into
//! the network game's settings (`dword_75ACE8` .. `byte_75AE87`): 91 dwords (the car,
//! the driving aids and the HUD's switches, the controls, the co-driver, the view; the
//! dword at `dword_75AD8C` twice, the second one kept), 12 bytes from +0x1C, 0x20 bytes from
//! +0x24C to `dword_75B06C`; and the record's +0xD0 made 1.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! A null `g_players[0]` (the original reads through it) is refused. Caller:
//! `net::start_link`. Shadow mode: both run, the settings and the record compared.
//! Self-test: a copy against the Rust version, on a test record put in `g_players[0]`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::net::lobby::{Part, a, g, movsb};
use rc2k_core::net::settings::{self, Copy};

const PLAYERS: usize = 0x75647C; // g_players
const SETTINGS: usize = 0x75ACE8; // dword_75ACE8
/// The Rust version: the registers it leaves (the entry's).
pub fn take_player_settings(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let record = g(PLAYERS);
    if record == 0 {
        return Err(BadInput("no record for player 1").into());
    }
    // Converted: `rc2k_core::net::settings` (`copies`, `TAKEN_MARK`).
    unsafe { ((record + settings::TAKEN_MARK) as *mut u32).write_unaligned(1) };
    let to = a(SETTINGS);
    for copy in settings::copies() {
        match copy {
            Copy::Dword { to: at, from } => {
                let v = unsafe { ((record + from) as *const u32).read_unaligned() };
                unsafe { ((to + at) as *mut u32).write_unaligned(v) };
            }
            Copy::Bytes { to: at, from, len } => movsb(to + at, record + from, len),
        }
    }
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        take_player_settings(r, &mut crate::net::lobby::game_part)
    })
}
