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
//!      `run_event_races`)
//!
//! Callers: the event drivers. The steps are their Rust versions; the races and the front
//! end through `part`, in the game their Rust versions. Ungated (the races). Refuses fewer
//! than 0xB2 drivers or fewer than four still running among them (the drop-out loop would
//! run on), a missing player for the mode, and what the steps refuse, before anything.
//! Shadow mode runs the original alone (it runs races, and logs). Self-test:
//! a copy whose races and front-end call go to stand-ins, against the Rust version,
//! by mode and result; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::standings::*;
use rc2k_core::menu::event_races::{self as races, PICKED, Step, Var};

const GOES_ON_RVA: usize = rva(0x75592C); // dword_75592C
const CHEAT_RVA: usize = rva(0x75ACF0); // dword_75ACF0
const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const KEPT_RVA: usize = rva(0x7554C8); // dword_7554C8
const RESET_ONLY_RVA: usize = rva(0x755250); // dword_755250
const SUMMARY_RVA: usize = rva(0x7556F4); // dword_7556F4
const RACE_INFO_RVA: usize = rva(0x75ACD0); // dword_75ACD0, dword_75ACD4
const MODE_RVA: usize = rva(0x755988); // dword_755988
const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const RESULT_RVA: usize = rva(0x75B028); // dword_75B028
const RESULT_2_RVA: usize = rva(0x75B098); // dword_75B098
const RACE_KEPT_RVA: usize = rva(0x75ACE4); // dword_75ACE4

/// A call by place: 1 `race::single_rally_race`, 2 `menu::reload_front_end`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The game for `rc2k_core::menu::event_races`: the variables at their addresses, the
/// steps' Rust versions on the registers carried through (`g`), the race and the front end
/// through `part`.
struct Game<'a> {
    g: Regs,
    part: Part<'a>,
    first: u32,
    rec: u32,
    pack: u32,
}

fn address(v: Var) -> usize {
    match v {
        Var::GoesOn => GOES_ON_RVA,
        Var::CheatSwitch => CHEAT_RVA,
        Var::Cheats => CHEATS_RVA,
        Var::Kept => KEPT_RVA,
        Var::RaceKept => RACE_KEPT_RVA,
        Var::Current => CURRENT_RVA,
        Var::Result => RESULT_RVA,
        Var::Result2 => RESULT_2_RVA,
    }
}

impl races::EventRaces for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        unsafe { game_read(address(v)) }
    }

    fn set(&mut self, v: Var, value: u32) {
        unsafe { game_write(address(v), value) };
    }

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        let g = &mut self.g;
        unsafe {
            match s {
                Step::CpuTimes => {
                    crate::menu::calc_cpu_times::calc_cpu_times(g)?;
                    g.ecx = 0;
                }
                Step::ResetPlayerTimes => {
                    crate::menu::reset_player_times::reset_player_times(g)?;
                }
                Step::StandingsFromDrivers => {
                    crate::menu::standings_from_drivers::standings_from_drivers(g)?;
                }
                Step::SortByTime => {
                    crate::menu::sort_standings_by_time::sort_standings_by_time(g)?;
                }
                Step::CopySummary => {
                    crate::menu::copy_standings_summary::copy_standings_summary(g)?;
                }
                Step::RaceInfo => {
                    let (rec, pack) = (self.rec, self.pack);
                    g.eax = game_read(SUMMARY_RVA);
                    game_write(RACE_INFO_RVA, g.eax);
                    g.eax = game_read(INDEX_RVA);
                    g.ebx = game_read(SECTION_RVA);
                    g.edi = pack;
                    g.esi = pack;
                    g.ebx = get(rec + 4).wrapping_mul(0x40);
                    g.ecx = get(rec + 8);
                    g.edx = crate::records::trakpak_entry::loaded_pack_id();
                    g.esi = game_addr(RACE_INFO_RVA) as u32;
                    g.eax = get(pack + 0x34);
                    game_write(RACE_INFO_RVA + 4, g.eax);
                }
                Step::Race => {
                    g.ebx = 1;
                    (self.part)(1, g)?;
                }
                Step::ReloadFrontEnd => (self.part)(2, g)?,
                Step::PlayerEntered { next } => {
                    *g = crate::menu::player_entered::player_entered(&Regs {
                        eax: 1,
                        ecx: next,
                        ..*g
                    })?;
                }
                Step::ClearPlayers288 => {
                    crate::menu::clear_players_288::clear_players_288(g)?;
                }
            }
        }
        Ok(())
    }

    fn below(&mut self, n: u32) -> Result<u32, Refused> {
        Ok(unsafe { crate::math::random_below::random_below(n)? })
    }

    fn slow(&mut self, k: u32, added: u32) {
        self.g.edi = self.first + k * DRIVER;
        self.g.eax = added;
        let edi = self.g.edi;
        put(edi + 0xC, get(edi + 0xC).wrapping_add(added));
        put(edi + 0x10, get(edi + 0x10).wrapping_add(added));
    }

    fn running(&mut self, k: u32) -> bool {
        self.g.eax = k * DRIVER;
        self.g.edi = self.first + self.g.eax;
        get(self.g.edi + 0x24) == races::RUNNING
    }

    fn drop_out(&mut self) {
        put(self.g.edi + 0x24, races::DROPPED);
        put(self.g.edi + 0xC, races::DROPPED_TIME);
        put(self.g.edi + 0x10, races::DROPPED_TIME);
    }

    fn takes_part(&mut self, i: u32) -> bool {
        self.g.edi = player(i);
        get(self.g.edi + 0xD0) == races::TAKES_PART
    }

    fn fourth_out(&mut self) {
        self.g.edi = player(3);
        put(self.g.edi + 0x88, races::FOURTH_TIME);
        put(self.g.edi + 0xD0, races::FOURTH_ENTRY);
    }

    fn keep(&mut self, value: u32) {
        self.g.eax = value;
    }

    fn hand_kept(&mut self, value: u32) {
        self.g.edx = value;
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_event_races(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let drivers = entries(game_addr(DRIVERS_RVA) as u32, DRIVER)?;
    if drivers.len() < PICKED as usize
        || drivers[..PICKED as usize]
            .iter()
            .filter(|&&d| get(d + 0x24) == 1)
            .count()
            < 4
    {
        return Err(BadInput("fewer than 0xB2 drivers, or fewer than four still running").into());
    }
    let mode: u32 = unsafe { game_read(MODE_RVA) };
    if let Some((n, _)) = races::players_of(mode)
        && (0..n).any(|i| player(i) == 0)
    {
        return Err(BadInput("a player of the event's mode missing").into());
    }
    let reset_only = unsafe { game_read::<u32>(RESET_ONLY_RVA) } == 1;
    let rec = crate::menu::calc_cpu_times::record()?;
    crate::menu::calc_cpu_times::check(rec)?;
    let pack = unsafe { crate::records::trakpak_entry::record_entry(rec as *const u8) }?;
    let mut game = Game {
        g: *entry,
        part,
        first: drivers[0],
        rec,
        pack,
    };
    // Converted: `rc2k_core::menu::event_races::run_event_races`.
    races::run_event_races(&mut game, mode, reset_only)?;
    Ok(game.g)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => {
                *r = crate::race::single_rally::single_rally_race(
                    r,
                    &mut crate::race::single_rally::game_part,
                )?
            }
            _ => {
                use crate::menu::reload_front_end as f;
                *r = f::reload_front_end(r)?
            }
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { run_event_races(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
