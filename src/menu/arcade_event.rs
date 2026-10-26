//! `menu::arcade_event` (0x487E2C): the arcade event from the car choice on, a level after
//! another (`dword_7587EC`, set by its five starters `menu::arcade_level_1` ..).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`) but as `show_in_game_options` leaves them
//!
//! `menu::clear_players_284`; player 0's +0xD0 1 (edi it); `dword_7587F4` 0;
//! `reset_cars_750724`; `dword_7587A8`, `dword_758798`, `dword_7587B8`, `dword_758768` 0,
//! `dword_755984` 1, `dword_755988` 0x1D, `dword_75555D` 0, the stage `dword_7587F0` 1. A
//! level: the current player 0; the car list (esi `off_75A042`, `secret_car_list`) and
//! `select_car_loop` (with `dword_758806` 1 straight to the end, without `dword_755980`);
//! `take_shown_car`. A stage: `dword_7587F4` 0; `arcade_next_stage_loop` (a "yes" ends);
//! `take_arcade_stage_best`; `dword_75AFF4` the stage, `dword_75AFF8` the level,
//! `dword_75ACF0` 1 (0 with cheat bit 0x12); eax 0 and `service::repair_car`; ebx 2 and
//! the race `race::single_rally_race`; with `dword_75B028` -1 `dword_758768` 0 and
//! `dword_7587F4` 1, else `dword_758758` = `dword_75AFFC` and `dword_758768` =
//! `dword_75B000` (eax it); the current player 0; `score_arcade_stage`. Unless
//! `dword_7587F4` is 1: `arcade_results_loop` (a "yes" ends); ebx the level, edi the
//! stage's record, eax the stage x 0xA4, `keep_arcade_stage_best`; unless `dword_7587F4`
//! is 1, `check_arcade_stage_time` and with `dword_755918` 1 `clear_arcade_time_marks`,
//! `enter_arcade_stage_time`, `arcade_best_times_loop` (a "yes" ends); the stage one
//! more. Past 6: `check_arcade_level_score` and with `dword_755918` 1
//! `clear_arcade_score_marks`, `enter_arcade_level_score`, `best_scores_48a69e_loop` (a
//! "yes" ends); then (also straight after a failed race) `arcade_level_end`; with
//! `dword_7587F4` 1 the retry below; at level 5 the end; else `review_options_4882e2_loop`
//! (ended by `dword_755569` 1 or `dword_755565` not 1), the level one more, the stage 1,
//! `dword_7587B8`, `dword_758798`, `dword_7587A8` 0 and the next level. Not past 6: with
//! `dword_7587F4` 1 the retry: `review_options_48861d_loop` (ended likewise), the three
//! totals 0, the stage 1, player 0's +0xD0 1, `dword_7587F4` 0 and the stage again; else
//! `arcade_review_options_loop` (ended by `dword_755569` or `dword_75555D` 1) and the next
//! stage. The end: `dword_755980` 1; after the `popa` `dword_755569`, `dword_755565` 0 and
//! `show_in_game_options`. `dword_755918` is never cleared here (the original's slip?
//! the checks only set it).
//!
//! The screens and the race are called at the game's addresses (their hooks choose); the
//! small steps are their Rust versions. Refuses (before anything) a missing player 0.
//! Ungated thunk; shadow mode runs the original alone. Manual patch with a private copy
//! (a call at the second byte; its 26 calls relocated). Self-test: a copy whose calls go
//! to stand-ins that answer by scripts (the screens' choices, the race's result), against
//! the Rust version on the same answers.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::parts::{Part, parted_entry};
use crate::menu::player::player_at;
use rc2k_core::menu::arcade_event::{self as arcade, Hand, Step, Var};

// The call sites (26: `secret_car_list` is called from two). 0x323, the call of
// `show_in_game_options` after the `popa`, was missing until 2026-10-20: the private copy
// called 0x027C7DB7 when shadow mode left the arcade ("Branch to Alpha Centauri").
/// The callees, each once (the parts' numbers).
pub const CALLEES: [usize; 25] = [
    0x4A9679, // 0 menu::clear_players_284
    0x46FC52, // 1 menu::reset_cars_750724
    0x480D35, // 2 menu::secret_car_list
    0x47AC0C, // 3 menu::select_car_loop
    0x47B29D, // 4 menu::take_shown_car
    0x48980C, // 5 menu::arcade_next_stage_loop
    0x48ADF8, // 6 menu::take_arcade_stage_best
    0x486F63, // 7 service::repair_car
    0x4843EC, // 8 race::single_rally_race
    0x48A383, // 9 menu::score_arcade_stage
    0x4891EF, // 10 menu::arcade_results_loop
    0x488213, // 11 menu::keep_arcade_stage_best
    0x488280, // 12 menu::check_arcade_stage_time
    0x48AD30, // 13 menu::clear_arcade_time_marks
    0x489F76, // 14 menu::enter_arcade_stage_time
    0x48A3F4, // 15 menu::arcade_best_times_loop
    0x488250, // 16 menu::check_arcade_level_score
    0x48AD08, // 17 menu::clear_arcade_score_marks
    0x48A03F, // 18 menu::enter_arcade_level_score
    0x48A69E, // 19 menu::best_scores_48a69e_loop
    0x48AD73, // 20 menu::arcade_level_end
    0x4882E2, // 21 menu::review_options_4882e2_loop
    0x48861D, // 22 menu::review_options_48861d_loop
    0x488958, // 23 menu::arcade_review_options_loop
    0x46FBE3, // 24 menu::show_in_game_options
];

fn g(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}
fn p(a: usize, v: u32) {
    unsafe { game_write(rva(a), v) }
}
fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The game for `rc2k_core::menu::arcade_event`: the variables at their addresses, the
/// steps through `part` with the registers the original gives them (one set carried
/// through).
struct Game<'a> {
    entry: Regs,
    r: Regs,
    player0: u32,
    part: Part<'a>,
    left: Option<Regs>,
}

fn address(v: Var) -> usize {
    match v {
        Var::Failed => 0x7587F4,
        Var::TotalA8 => 0x7587A8,
        Var::Total798 => 0x758798,
        Var::TotalB8 => 0x7587B8,
        Var::Figure768 => 0x758768,
        Var::Figure758 => 0x758758,
        Var::PlayerCount => 0x755984,
        Var::EventMode => 0x755988,
        Var::Leave => 0x75555D,
        Var::Ended565 => 0x755565,
        Var::Ended569 => 0x755569,
        Var::Stage => 0x7587F0,
        Var::Level => 0x7587EC,
        Var::Current => 0x75598C,
        Var::CarLeft => 0x758806,
        Var::Answer => 0x75524C,
        Var::RaceStage => 0x75AFF4,
        Var::RaceLevel => 0x75AFF8,
        Var::CheatSwitch => 0x75ACF0,
        Var::Cheats => 0x75870C,
        Var::RaceResult => 0x75B028,
        Var::RaceFigureFFC => 0x75AFFC,
        Var::RaceFigure000 => 0x75B000,
        Var::Qualifies => 0x755918,
        Var::Done => 0x755980,
        Var::Player0Entry => unreachable!("player 0's field is not at an address"),
    }
}

impl arcade::Arcade for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        g(address(v))
    }

    fn set(&mut self, v: Var, value: u32) {
        match v {
            Var::Player0Entry => {
                self.r.edi = self.player0;
                put(self.player0 + 0xD0, value);
            }
            _ => p(address(v), value),
        }
    }

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        match s {
            Step::SecretCarList => self.r.esi = game_addr(rva(0x75A042)) as u32,
            Step::RepairCar => self.r.eax = 0,
            Step::Race => self.r.ebx = 2,
            Step::ShowInGameOptions => {
                let mut out = self.entry;
                (self.part)(s as u32, &mut out)?;
                self.left = Some(out);
                return Ok(());
            }
            _ => {}
        }
        (self.part)(s as u32, &mut self.r)
    }

    fn hand(&mut self, h: Hand) -> Result<(), Refused> {
        match h {
            Hand::Kept(v) => self.r.eax = v,
            Hand::StageRecord => {
                self.r.ebx = g(0x7587EC);
                self.r.edi = g(0x73555A + 4 * self.r.ebx as usize);
                self.r.eax = g(0x7587F0).wrapping_mul(0xA4);
                self.r.edi = self.r.edi.wrapping_add(self.r.eax);
            }
        }
        Ok(())
    }
}

/// The Rust version: the registers it leaves.
pub unsafe fn arcade_event(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let player0 = player_at(0);
    if player0 == 0 {
        return Err(BadInput("no player 0").into());
    }
    let mut game = Game {
        entry: *entry,
        r: *entry,
        player0,
        part,
        left: None,
    };
    // Converted: `rc2k_core::menu::arcade_event::arcade_event`.
    arcade::arcade_event(&mut game)?;
    Ok(game.left.unwrap_or(*entry))
}

/// The game's callees: the small steps' Rust versions, the screens and the race at the
/// game's addresses.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::menu::arcade_steps as s;
    unsafe {
        match place {
            0 => crate::menu::clear_players_284::clear_players_284()?,
            1 => *r = crate::menu::reset_cars_750724::reset_cars_750724(r)?,
            2 => s::secret_car_list().apply(r),
            4 => crate::menu::take_shown_car::take_shown_car(r.esi)?,
            6 => s::take_arcade_stage_best()?,
            7 => crate::service::repair_parts::repair_car()?,
            9 => s::score_arcade_stage()?,
            11 => s::keep_arcade_stage_best(r.edi)?,
            12 => s::check_arcade_stage_time()?,
            13 => s::clear_arcade_time_marks()?,
            14 => s::enter_arcade_stage_time()?,
            16 => s::check_arcade_level_score()?,
            17 => s::clear_arcade_score_marks()?,
            18 => s::enter_arcade_level_score()?,
            20 => {
                *r = crate::menu::arcade_level_end::arcade_level_end(
                    r,
                    &mut crate::menu::arcade_level_end::game_part,
                )?
            }
            24 => crate::menu::in_game_options_items::show_in_game_options(),
            _ => crate::calls::call(rva(CALLEES[place as usize]), r),
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    parted_entry(r, &|r, part| unsafe { arcade_event(r, part) }, game_part)
}
