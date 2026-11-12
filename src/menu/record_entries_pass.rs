//! `menu::record_entries_pass` (0x4A3BE0): the current event's record entries cleared
//! (`menu::clear_record_entries`), then each counted player (`event.current_player` 0 ..
//! `event.player_count`, the original's `do .. while`) that beat the target time
//! (`menu::beat_target_time`, `event.time_qualifies`) entered (`menu::enter_record_entry`),
//! `event.any_time_qualified` 1 once one is (0 first).
//!
//! Callers: the event drivers. Refuses what the steps refuse and
//! a player count of 0 or past 4 or a player missing, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::standings::*;
// hhmm
// g_cheats
/// The steps of `rc2k_core::menu::bests::record_entries`.
struct Entries<'a>(&'a Menus, &'a System, &'a crate::race::owner::Race);

impl rc2k_core::menu::bests::RecordEntries for Entries<'_> {
    type Error = Refused;

    fn clear(&mut self) -> Result<(), Refused> {
        let screens = &self.0.screens;
        crate::menu::clear_record_entries::clear_record_entries(screens, self.1)
    }

    fn set_entered(&mut self, on: bool) {
        self.0.screens.event.any_time_qualified.set(on);
    }

    fn set_current(&mut self, n: u32) {
        self.0.screens.event.current_player.set(n);
    }

    fn beats(&mut self, n: u32) -> Result<bool, Refused> {
        let _ = n;
        crate::menu::beat_target_time::beat_target_time(self.0, self.1)?;
        Ok({ self.0.screens.event.time_qualifies.get() } == 1)
    }

    fn enter(&mut self, n: u32) -> Result<(), Refused> {
        let menus = self.0;
        let _ = n;
        crate::menu::enter_record_entry::enter_record_entry(menus, self.1, self.2)
    }
}

pub fn record_entries_pass(
    menus: &Menus,
    system: &System,
    race: &crate::race::owner::Race,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    let players = counted_players(menus)?;
    current_event(screens, system)?;
    if any_unnamed(&players)? {
        return Err(BadInput("a player without a name").into());
    }
    // The logic is in `rc2k_core::menu::bests::record_entries`.
    rc2k_core::menu::bests::record_entries(players.len() as u32, &mut Entries(menus, system, race))
}
