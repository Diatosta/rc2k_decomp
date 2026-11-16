//! `menu::run_event_races`: an event's races. The computer drivers' times are made
//! (`menu::calc_cpu_times`); eight drivers picked by lot (of the first 0xB2) get 0x4844 plus a
//! random number under 3000 added to their +0xC and +0x10, and one to four drivers by lot still
//! running (+0x24 1) drop out (+0x24 4, +0xC and +0x10 0x36EE8A). With `reset_times` set the
//! players' times are reset; otherwise standings table A is built from the drivers, sorted by time,
//! and the summary kept. Then by the event mode (`event_kind`): 9 / 0xE one race, 0xA / 0xF the
//! same with the kept value, 0xB .. 0x12 each player taking part races in turn; a result of -1 ends
//! the event, -2 resets the players' times and sets the front end up again. Last, the players'
//! +0x288 is cleared (`menu::clear_service_spent`).
//!
//! Callers: the event drivers. Refuses fewer than 0xB2 drivers or fewer than four still running
//! among them (the drop-out draw would never end), a missing player for the mode, and what the
//! steps refuse, before anything.

use crate::errors::{BadInput, Refused};
use crate::memory::session::DwordLike;
use crate::menu::drivers_store::{Driver, IN_EVENT};
use crate::menu::event_flow::Fault;
use crate::menu::owner::{Cars, Play, Screens};
use crate::menu::standings::*;
use crate::owners::Owners;
use crate::race::owner::Race;
use rc2k_core::menu::event_races::{self as races, PICKED, Step, Var};

/// The game for `rc2k_core::menu::event_races`: the variables at their addresses, the
/// steps, the race and the front end.
struct Game<'a> {
    cars: &'a Cars,
    screens: &'a Screens,
    play: &'a Play,
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
fn cell<'a>(screens: &'a Screens, play: &'a Play, race: &'a Race, v: Var) -> &'a dyn DwordLike {
    match v {
        Var::CheatSwitch => &race.shared.state.settings.direction,
        Var::RaceKept => &race.shared.state.settings.kept,
        Var::Result => &race.shared.state.results.results[0].result,
        Var::Result2 => &race.shared.state.results.results[1].result,
        Var::GoesOn => &screens.event.event_goes_on,
        Var::Cheats => &play.cheats.cheats,
        Var::Kept => &screens.flow.kept,
        Var::Current => &screens.event.current_player,
    }
}

impl races::EventRaces for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        cell(self.screens, self.play, self.owners.race, v).get()
    }

    fn set(&mut self, v: Var, value: u32) {
        cell(self.screens, self.play, self.owners.race, v).set(value);
    }

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        let race = self.owners.race;
        let (screens, play) = (self.screens, self.play);
        let tables = &self.owners.saves.best_tables;
        self.fault.check()?;
        match s {
            Step::CpuTimes => crate::menu::calc_cpu_times::calc_cpu_times(
                self.cars,
                screens,
                self.owners.system,
                self.owners.race,
                tables,
            )?,
            Step::ResetPlayerTimes => {
                crate::menu::reset_player_times::reset_player_times(screens, play)?;
            }
            Step::StandingsFromDrivers => {
                crate::menu::standings_from_drivers::standings_from_drivers(
                    crate::strings::append_string_by_id::StringOwners::of(self.owners),
                    self.cars,
                    tables,
                )?;
            }
            Step::SortByTime => {
                crate::menu::sort_standings_by_time::sort_standings_by_time(screens, play, tables)?;
            }
            Step::CopySummary => {
                crate::menu::copy_standings_summary::copy_standings_summary(screens, tables)?;
            }
            Step::RaceInfo => {
                // The summary's variable: 1 while it is there (the race reads it:
                // `RACE_STATE.settings.rivals`).
                race.shared
                    .state
                    .settings
                    .rivals
                    .set(play.champ_view.summary.get());
                race.shared.state.settings.stage.set(
                    self.pack
                        .stage(&self.owners.system.records().stage_tables)?,
                );
            }
            Step::Race { next } => {
                crate::race::single_rally_race::single_rally_race(self.owners, u32::from(next), 1)?;
            }
            Step::ReloadFrontEnd => crate::menu::reload_front_end::reload_front_end(self.owners)?,
            Step::ClearServiceSpent => {
                crate::menu::clear_service_spent::clear_service_spent(play)?;
            }
        }

        Ok(())
    }

    fn below(&mut self, n: u32) -> Result<u32, Refused> {
        Ok(self.owners.race.shared.random.random_below(n)?)
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
        player_in(self.play, i).get().entry == races::TAKES_PART
    }

    fn fourth_out(&mut self) {
        player_in(self.play, 3).update(|q| {
            q.time = races::FOURTH_TIME;
            q.entry = races::FOURTH_ENTRY;
        });
    }
}

pub fn run_event_races(o: Owners<'static>) -> Result<(), Refused> {
    let (cars, screens, play) = (&o.menus.cars, &o.menus.screens, &o.menus.play);
    let drivers = drivers_of(cars)?;
    let few_running = drivers.len() < PICKED as usize
        || drivers[..PICKED as usize]
            .iter()
            .filter(|d| d.in_event.get() == IN_EVENT)
            .count()
            < 4;
    if few_running {
        return Err(BadInput("fewer than 0xB2 drivers, or fewer than four still running").into());
    }
    let mode: u32 = screens.event.event_kind.get();
    if let Some((n, _)) = races::players_of(mode)
        && n > 4
    {
        return Err(BadInput("a player of the event's mode missing").into());
    }
    let reset_only = { play.champ_view.reset_times.get() } == 1;
    let rec = crate::menu::calc_cpu_times::record(screens, o.system, &o.saves.best_tables)?;
    crate::menu::calc_cpu_times::check(cars, screens, o.system, o.race, rec)?;
    let pack = crate::records::trakpak_entry::track_of(&o.system.records().stage_tables, rec)?;
    let mut game = Game {
        cars,
        screens,
        play,
        owners: o,
        drivers,
        pack,
        asked: None,
        fault: Fault::default(),
    };
    let done = races::run_event_races(&mut game, mode, reset_only);
    game.fault.check()?;
    done
}
