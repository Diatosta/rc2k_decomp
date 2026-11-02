//! `menu::run_event_races` (0x473126): an event's races. `dword_75592C` 1 (the event goes
//! on), `dword_75ACF0` the cheat bit 0x12, `dword_7554C8` 0; the computer drivers' times
//! (`menu::calc_cpu_times`); eight drivers picked by lot (of the first 0xB2) get 0x4844
//! plus a random number under 3000 added to their +0xC and +0x10; one to four drivers by
//! lot still running (+0x24 1) drop out (+0x24 4, +0xC and +0x10 0x36EE8A). With
//! `dword_755250` 1 the players' times are reset (`menu::reset_player_times`); otherwise
//! standings table A from the drivers (`menu::standings_from_drivers`), sorted by time,
//! the summary copied (`menu::copy_standings_summary`), `dword_75ACD0` the summary table
//! and `dword_75ACD4` the record's track pack entry's +0x34; then by the event mode
//! (`dword_755988`):
//! - 9, 0xE: one race (`race::single_rally_race`, eax 0, ebx 1); a result
//!   (`dword_75B028`) of -1 ends the event (`dword_75592C` 0).
//! - 0xA, 0xF: the same with `dword_75ACE4` `dword_7554C8` (0xA: before and after), the
//!   result and `dword_75B098` both -1 ending it.
//! - 0xB, 0x10 (two players), 0xC, 0x11 (three), 0xD, 0x12 (four): each player
//!   (`dword_75598C`) taking part (+0xD0 1) races, eax 1 when the next one takes part
//!   (`menu::player_entered`), 0 for the last; a result of -2 resets the players' times,
//!   sets the front end up again (`menu::reload_front_end`) and stops; one not taking
//!   part counts -1; all -1 ends the event (0xD also gives the fourth player +0x88
//!   0x36EE80 and +0xD0 6).
//!
//! Then the players' +0x288 cleared (`menu::clear_players_288`).
//!
//! In:  nothing
//! Out: no `pusha`: eax, ecx, edx, ebx, esi and edi as the original leaves them (see
//! `run_event_races`)
//!
//! Callers: the event drivers. The steps are their Rust versions. Refuses fewer than 0xB2 drivers
//! or fewer than four still running among them (the drop-out loop would run on), a missing player
//! for the mode, and what the steps refuse, before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::cheats_store::CHEAT_VARS;
use crate::menu::event_flow::Fault;
use crate::menu::event_flow::Flow;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::standings::*;
use crate::race::state_store::RACE_STATE;
use crate::session::Dword;
use rc2k_core::menu::event_races::{self as races, PICKED, Step, Var};

const GOES_ON: &Dword = &EVENT_STATE.event_goes_on; // dword_75592C
const CHEATS: &Dword = &CHEAT_VARS.cheats; // g_cheats
const KEPT: &Dword = &SCREEN_FLOW.kept; // dword_7554C8
const CURRENT: &Dword = &EVENT_STATE.current_player; // dword_75598C

/// The game for `rc2k_core::menu::event_races`: the variables at their addresses, the
/// steps' Rust versions on the registers carried through (`g`), the race and the front end.
struct Game<'a> {
    g: &'a mut Flow,
    first: u32,
    rec: u32,
    pack: u32,
    /// A refused read or write of the core's variables (checked at each step and at the end).
    fault: Fault,
}

/// The variables that are in the physics block.
fn cell(v: Var) -> &'static Dword {
    match v {
        Var::CheatSwitch => &RACE_STATE.settings.direction,
        Var::RaceKept => &RACE_STATE.settings.kept,
        Var::Result => &RACE_STATE.results.results[0].result,
        Var::Result2 => &RACE_STATE.results.results[1].result,
        Var::GoesOn => GOES_ON,
        Var::Cheats => CHEATS,
        Var::Kept => KEPT,
        Var::Current => CURRENT,
    }
}

impl races::EventRaces for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        cell(v).get()
    }

    fn set(&mut self, v: Var, value: u32) {
        cell(v).set(value);
    }

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        self.fault.check()?;
        let g = &mut *self.g;
        match s {
            Step::CpuTimes => {
                crate::menu::calc_cpu_times::calc_cpu_times()?;
                g.c.measure_x = 0;
            }
            Step::ResetPlayerTimes => {
                crate::menu::reset_player_times::reset_player_times()?;
            }
            Step::StandingsFromDrivers => {
                crate::menu::standings_from_drivers::standings_from_drivers()?;
            }
            Step::SortByTime => {
                crate::menu::sort_standings_by_time::sort_standings_by_time()?;
            }
            Step::CopySummary => {
                crate::menu::copy_standings_summary::copy_standings_summary()?;
            }
            Step::RaceInfo => {
                let (rec, pack) = (self.rec, self.pack);
                // The summary's variable: 1 while it is there (the race reads it:
                // `RACE_STATE.settings.rivals`).
                g.c.arg = CHAMP_VIEW.summary.get();
                RACE_STATE.settings.rivals.set(g.c.arg);
                g.c.arg = INDEX_RVA.get();
                g.c.palette = SECTION_RVA.get();
                g.record = pack;
                g.c.cars = pack;
                g.c.palette = get(rec + 4)?.wrapping_mul(0x40);
                g.c.measure_x = get(rec + 8)?;
                g.c.clock = crate::records::trakpak_entry::loaded_pack_id();
                g.c.cars = RACE_STATE.settings.rivals.addr();
                g.c.arg = get(pack + 0x34)?;
                RACE_STATE.settings.stage.set(g.c.arg);
            }
            Step::Race => {
                g.c.palette = 1;
                crate::race::single_rally_race::single_rally_race(g.c.arg, 1)?;
            }
            Step::ReloadFrontEnd => crate::menu::reload_front_end::reload_front_end()?,
            Step::PlayerEntered { next } => {
                let (record, value) = crate::menu::player_entered::player_entered(next, 1)?;
                (g.c.measure_x, g.record, g.c.arg) = (next, record, value);
            }
            Step::ClearPlayers288 => {
                crate::menu::clear_players_288::clear_players_288()?;
            }
        }

        Ok(())
    }

    fn below(&mut self, n: u32) -> Result<u32, Refused> {
        Ok(crate::math::random_below::random_below(n)?)
    }

    fn slow(&mut self, k: u32, added: u32) {
        self.g.record = self.first + k * DRIVER;
        self.g.c.arg = added;
        let edi = self.g.record;
        let fault = &self.fault;
        fault.put(edi + 0xC, fault.get(edi + 0xC).wrapping_add(added));
        fault.put(edi + 0x10, fault.get(edi + 0x10).wrapping_add(added));
    }

    fn running(&mut self, k: u32) -> bool {
        self.g.c.arg = k * DRIVER;
        self.g.record = self.first + self.g.c.arg;
        self.fault.get(self.g.record + 0x24) == races::RUNNING
    }

    fn drop_out(&mut self) {
        self.fault.put(self.g.record + 0x24, races::DROPPED);
        self.fault.put(self.g.record + 0xC, races::DROPPED_TIME);
        self.fault.put(self.g.record + 0x10, races::DROPPED_TIME);
    }

    fn takes_part(&mut self, i: u32) -> bool {
        self.g.record = player(i).addr();
        self.fault.get(self.g.record + 0xD0) == races::TAKES_PART
    }

    fn fourth_out(&mut self) {
        self.g.record = player(3).addr();
        self.fault.put(self.g.record + 0x88, races::FOURTH_TIME);
        self.fault.put(self.g.record + 0xD0, races::FOURTH_ENTRY);
    }

    fn keep(&mut self, value: u32) {
        self.g.c.arg = value;
    }

    fn hand_kept(&mut self, value: u32) {
        self.g.c.clock = value;
    }
}

/// The Rust version (see the header). `f` the registers carried in and out (what it
/// leaves is the last step's).
pub fn run_event_races(f: &mut Flow) -> Result<(), Refused> {
    let drivers = game_entries(crate::menu::drivers_store::DRIVERS.first(), DRIVER)?;
    let fault = Fault::default();
    let few_running = drivers.len() < PICKED as usize
        || drivers[..PICKED as usize]
            .iter()
            .filter(|&&d| fault.get(d + 0x24) == 1)
            .count()
            < 4;
    fault.check()?;
    if few_running {
        return Err(BadInput("fewer than 0xB2 drivers, or fewer than four still running").into());
    }
    let mode: u32 = EVENT_STATE.event_kind.get();
    if let Some((n, _)) = races::players_of(mode)
        && n > 4
    {
        return Err(BadInput("a player of the event's mode missing").into());
    }
    let reset_only = { MENU_PAGES.reset_times.get() } == 1;
    let rec = crate::menu::calc_cpu_times::record()?;
    crate::menu::calc_cpu_times::check(rec)?;
    let pack = crate::records::trakpak_entry::record_entry(rec)?;
    let mut game = Game {
        g: f,
        first: drivers[0],
        rec,
        pack,
        fault: Fault::default(),
    };
    // Converted: `rc2k_core::menu::event_races::run_event_races`.
    let done = races::run_event_races(&mut game, mode, reset_only);
    game.fault.check()?;
    done
}
