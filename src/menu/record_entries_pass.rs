//! `menu::record_entries_pass` (0x4A3BE0): the current event's record entries cleared
//! (`menu::clear_record_entries`), then each counted player (`dword_75598C` 0 ..
//! `dword_755984`, the original's `do .. while`) that beat the target time
//! (`menu::beat_target_time`, `dword_755918`) entered (`menu::enter_record_entry`),
//! `dword_755920` 1 once one is (0 first).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. The steps are their Rust versions. Refuses what the steps refuse and
//! a player count of 0 or past 4 or a player missing, before anything.
use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::menu::standings::*;

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
// dword_7559EC
// hhmm
const ENTERED_RVA: usize = rva(0x755920); // dword_755920
const BEATEN_RVA: usize = rva(0x755918); // dword_755918
// g_cheats
/// The steps of `rc2k_core::menu::bests::record_entries`: their Rust versions.
struct Entries;

impl rc2k_core::menu::bests::RecordEntries for Entries {
    type Error = Refused;

    fn clear(&mut self) -> Result<(), Refused> {
        unsafe { crate::menu::clear_record_entries::clear_record_entries() }
    }

    fn set_entered(&mut self, on: u32) {
        unsafe { game_write(ENTERED_RVA, on) };
    }

    fn set_current(&mut self, n: u32) {
        unsafe { game_write(CURRENT_RVA, n) };
    }

    fn beats(&mut self, n: u32) -> Result<bool, Refused> {
        let _ = n;
        unsafe { crate::menu::beat_target_time::beat_target_time()? };
        Ok(unsafe { game_read::<u32>(BEATEN_RVA) } == 1)
    }

    fn enter(&mut self, n: u32) -> Result<(), Refused> {
        let _ = n;
        unsafe { crate::menu::enter_record_entry::enter_record_entry() }
    }
}

/// The Rust version (see the header).
pub unsafe fn record_entries_pass() -> Result<(), Refused> {
    let players = counted_players()?;
    current_event()?;
    if players.iter().any(|&p| get(p + 0x18) == 0) {
        return Err(BadInput("a player without a name").into());
    }
    // Converted: `rc2k_core::menu::bests::record_entries`.
    rc2k_core::menu::bests::record_entries(players.len() as u32, &mut Entries)
}
