//! `menu::run_event_races`: an event's races. `event_goes_on` is set to 1, the cheat bit 0x12
//! is copied for the race, the kept value is cleared; the computer drivers' times
//! (`menu::calc_cpu_times`); eight drivers picked by lot (of the first 0xB2) get 0x4844
//! plus a random number under 3000 added to their +0xC and +0x10; one to four drivers by
//! lot still running (+0x24 1) drop out (+0x24 4, +0xC and +0x10 0x36EE8A). With
//! `reset_times` set the players' times are reset (`menu::reset_player_times`); otherwise
//! standings table A from the drivers (`menu::standings_from_drivers`), sorted by time,
//! the summary copied (`menu::copy_standings_summary`), the race's summary table and the
//! record's track pack entry's +0x34 kept; then by the event mode (`event_kind`):
//! - 9, 0xE: one race (`race::single_rally_race`, eax 0, ebx 1); a result of -1 ends the
//!   event (`event_goes_on` cleared).
//! - 0xA, 0xF: the same with the kept value (0xA: before and after), the result and the
//!   second result both -1 ending it.
//! - 0xB, 0x10 (two players), 0xC, 0x11 (three), 0xD, 0x12 (four): each player
//!   (`current_player`) taking part (+0xD0 1) races, eax 1 when the next one takes part
//!   (`menu::player_entered`), 0 for the last; a result of -2 resets the players' times,
//!   sets the front end up again (`menu::reload_front_end`) and stops; one not taking
//!   part counts -1; all -1 ends the event (0xD also gives the fourth player +0x88
//!   0x36EE80 and +0xD0 6).
//!
//! Then the players' +0x288 cleared (`menu::clear_service_spent`).
//!
//! Callers: the event drivers. Refuses fewer than 0xB2 drivers
//! or fewer than four still running among them (the drop-out loop would run on), a missing player
//! for the mode, and what the steps refuse, before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::drivers_store::{Driver, IN_EVENT};
use crate::menu::event_flow::Fault;
use crate::menu::owner::Menus;
use crate::menu::standings::*;
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::session::DwordLike;
use rc2k_core::menu::event_races::{self as races, PICKED, Step, Var};

/// The game for `rc2k_core::menu::event_races`: the variables at their addresses, the
/// steps, the race and the front end.
struct Game<'a> {
    menus: &'a Menus,
    owners: Owners<'static>,
    /// The walked drivers.
    drivers: &'a [Driver],
    /// The stage's track pack entry.
    pack: crate::records::trakpak_entry::Track,
    /// The driver `running` last asked about (the one `drop_out` drops).
    asked: Option<&'a Driver>,
    /// A refused read or write of the core's variables (checked at each step and at the end).
    fault: Fault,
}

impl<'a> Game<'a> {
    /// Driver `k` of the walked ones; past them a refusal kept for the next check (the core
    /// picks under `PICKED`, which `run_event_races` checks the walk reaches).
    fn driver(&self, k: u32) -> Option<&'a Driver> {
        let d = self.drivers.get(k as usize);
        if d.is_none() {
            self.fault
                .ok::<()>(Err(BadInput("a driver past the walked ones")));
        }
        d
    }
}

/// The variables that are in the physics block.
fn cell<'a>(menus: &'a Menus, race: &'a Race, v: Var) -> &'a dyn DwordLike {
    match v {
        Var::CheatSwitch => &race.shared.state.settings.direction,
        Var::RaceKept => &race.shared.state.settings.kept,
        Var::Result => &race.shared.state.results.results[0].result,
        Var::Result2 => &race.shared.state.results.results[1].result,
        Var::GoesOn => &menus.screens.event.event_goes_on,
        Var::Cheats => &menus.play.cheats.cheats, // g_cheats
        Var::Kept => &menus.screens.flow.kept,
        Var::Current => &menus.screens.event.current_player,
    }
}

impl races::EventRaces for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        cell(self.menus, self.owners.race, v).get()
    }

    fn set(&mut self, v: Var, value: u32) {
        cell(self.menus, self.owners.race, v).set(value);
    }

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        let race = self.owners.race;
        let menus = self.menus;
        let screens = &menus.screens;
        self.fault.check()?;
        match s {
            Step::CpuTimes => crate::menu::calc_cpu_times::calc_cpu_times(
                self.menus,
                self.owners.system,
                self.owners.race,
            )?,
            Step::ResetPlayerTimes => {
                crate::menu::reset_player_times::reset_player_times(menus)?;
            }
            Step::StandingsFromDrivers => {
                crate::menu::standings_from_drivers::standings_from_drivers(menus)?;
            }
            Step::SortByTime => {
                crate::menu::sort_standings_by_time::sort_standings_by_time(menus)?;
            }
            Step::CopySummary => {
                crate::menu::copy_standings_summary::copy_standings_summary(screens)?;
            }
            Step::RaceInfo => {
                // The summary's variable: 1 while it is there (the race reads it:
                // `RACE_STATE.settings.rivals`).
                race.shared
                    .state
                    .settings
                    .rivals
                    .set(menus.play.champ_view.summary.get());
                race.shared.state.settings.stage.set(self.pack.stage()?);
            }
            Step::Race { next } => {
                crate::race::single_rally_race::single_rally_race(u32::from(next), 1)?;
            }
            Step::ReloadFrontEnd => crate::menu::reload_front_end::reload_front_end(self.owners)?,
            Step::ClearServiceSpent => {
                crate::menu::clear_service_spent::clear_service_spent(menus)?;
            }
        }

        Ok(())
    }

    fn below(&mut self, n: u32) -> Result<u32, Refused> {
        Ok(crate::math::random_below::random_below(n)?)
    }

    fn slow(&mut self, k: u32, added: u32) {
        if let Some(d) = self.driver(k) {
            d.time.set(d.time.get().wrapping_add(added));
            d.total.set(d.total.get().wrapping_add(added));
        }
    }

    fn running(&mut self, k: u32) -> bool {
        self.asked = self.driver(k);
        self.asked
            .is_some_and(|d| d.in_event.get() == races::RUNNING)
    }

    fn drop_out(&mut self) {
        if let Some(d) = self.asked {
            d.in_event.set(races::DROPPED);
            d.time.set(races::DROPPED_TIME);
            d.total.set(races::DROPPED_TIME);
        }
    }

    fn takes_part(&mut self, i: u32) -> bool {
        player(self.menus, i).get().entry == races::TAKES_PART
    }

    fn fourth_out(&mut self) {
        player(self.menus, 3).update(|q| {
            q.time = races::FOURTH_TIME;
            q.entry = races::FOURTH_ENTRY;
        });
    }
}

pub fn run_event_races(o: Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    let drivers = drivers(menus)?;
    let few_running = drivers.len() < PICKED as usize
        || drivers[..PICKED as usize]
            .iter()
            .filter(|d| d.in_event.get() == IN_EVENT)
            .count()
            < 4;
    if few_running {
        return Err(BadInput("fewer than 0xB2 drivers, or fewer than four still running").into());
    }
    let mode: u32 = menus.screens.event.event_kind.get();
    if let Some((n, _)) = races::players_of(mode)
        && n > 4
    {
        return Err(BadInput("a player of the event's mode missing").into());
    }
    let reset_only = { menus.tables.pages.reset_times.get() } == 1;
    let rec = crate::menu::calc_cpu_times::record(screens, o.system)?;
    crate::menu::calc_cpu_times::check(menus, o.race, rec)?;
    let pack = crate::records::trakpak_entry::track_of(rec)?;
    let mut game = Game {
        menus,
        owners: o,
        drivers,
        pack,
        asked: None,
        fault: Fault::default(),
    };
    // The logic is in `rc2k_core::menu::event_races::run_event_races`.
    let done = races::run_event_races(&mut game, mode, reset_only);
    game.fault.check()?;
    done
}
