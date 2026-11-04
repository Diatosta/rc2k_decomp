//! The arcade event's small steps (`menu::arcade_event`, 0x487E2C, and its callers; agent
//! E, 2026-10-19). Plain stores, all `pusha` .. `popa` unless said. What they share:
//! `dword_7587EC` the level (1 .. 5), `dword_7587F0` the stage (0 .. 6), the stage's record
//! `off_73555A[level]` + stage x 0xA4 (its +0x3C the course), the course's ten best times
//! (`dword_7556E0` + course x 0x618, entries of 0x9C, the tenth at +0x57C), the level's ten
//! best scores (`dword_7556DC` + level x 0x320, entries of 0x50, the tenth at +0x2D0), the
//! current player `dword_75598C` (`g_players`).
//! - `menu::keep_arcade_stage_best` (0x488213): edi the stage's record (the caller's): if
//!   its +0x30 is below the player's +0x94 (signed), +0x30 that, +0x74 / +0x78 the
//!   player's +4 / +8 and the 0x20 bytes at the player's +0x18 (a pointer) at +0x54.
//! - `menu::check_arcade_level_score` (0x488250): `dword_755918` 1 if the score
//!   `dword_7587B8` is at least the tenth best's +0x14 (signed).
//! - `menu::check_arcade_stage_time` (0x488280): `dword_755914` the player's time (+0x88);
//!   `dword_755918` 1 if it is at most the tenth best's +0x14 (signed).
//! - `menu::enter_arcade_stage_time` (0x489F76): the player's time in the tenth best
//!   (+0x14 its +0x88, +0x1C +0x90, +8 +4, +0xC +0x18, +0x10 +8, +0x50 `g_cheats`, +0x54
//!   / +0x58 `menu::sealed_player_time` of the course, +4 and +0x40 1, +0x20 the +0x18,
//!   0x40 bytes of the player's +0x20C at +0x5C, then 31 bytes of the name +0xC points
//!   at over +0x20), `dword_7559EC` the course first; then `sort_arcade_stage_times`.
//! - `menu::sort_arcade_stage_times` (0x48A0CE): nine times, from the tenth up: a pair
//!   whose lower time (+0x14) is at most the upper's (signed) exchanges +0xC, +0x14, +4,
//!   +8, +0x1C, +0x10, +0x18, +0x40, +0x50 .. +0x98 and the 31 bytes of the names at
//!   +0x20 (through `g_keptPlayerName`, which keeps the last upper name); either way the
//!   pair one up is next.
//! - `menu::enter_arcade_level_score` (0x48A03F): the score in the tenth best (+0x14
//!   `dword_7587B8`, or `dword_7587BC` for player 1; +0x1C, +8, +0xC, +0x10, +4, +0x40, +0x20
//!   as above, the name over +0x20); then `sort_arcade_level_scores`.
//! - `menu::sort_arcade_level_scores` (0x48A2AE): as the times' sort on entries of 0x50,
//!   exchanging when the lower score is at least the upper's: +0xC, +0x14, +4, +8, +0x1C,
//!   +0x10, +0x18, +0x40 and the names.
//! - `menu::score_arcade_stage` (0x48A383): the total before (`dword_7587A8` =
//!   `dword_7587B8`), the bonus `dword_758788` (player 0's +0x94), the place's points
//!   `dword_758778` = (`dword_7587F8` - `dword_758768` + 1) x 1000, their sum with
//!   `dword_758758` (`dword_758798`) and the total.
//! - `menu::clear_arcade_score_marks` (0x48AD08): +0x40 of the level's ten scores 0.
//! - `menu::clear_arcade_time_marks` (0x48AD30): +0x40 of the course's ten times 0.
//! - `menu::take_arcade_stage_best` (0x48ADF8): 0x40 bytes of the course's best's +0x5C
//!   into `unk_75B02C`.
//! - `menu::add_shown_car` (0x486BD2): the shown car (`off_75A12E[dword_755990]`) over
//!   the end mark of the list `off_75A042` (no new end mark: the original's slip, kept).
//! - `menu::remove_shown_car` (0x486BF5): the entries of `off_75A12E` after the shown one
//!   moved down one, up to and with the end mark; `saves::list_car_numbers`.
//! - `menu::secret_car_list` (0x480D35): with cheat bit 1 (`g_cheats`), esi `off_75A412`;
//!   the carry is the bit. Not `pusha`'d.
//!
//! Converted (2026-10-21): the entries' logic is `rc2k_core::menu::bests` and
//! `rc2k_core::menu::standings`; this file reads and writes the game's memory for it.
//!
//! Refusals (before anything): a level past 5, a stage past 6, a missing record or table
//! on the way, a current player past 3 or missing, a null name, a list without its end
//! in 0x28 (0x30 for `off_75A12E`) entries, a shown car past the list.

use crate::errors::{BadInput, Refused};
use crate::menu::arcade_store::ARCADE_GAME;
use crate::menu::cheats_store::CHEAT_VARS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::results_store::RESULT_FIGURES;
use crate::menu::standings::{bytes, table, with_kept_name};
use crate::race::state_store::RACE_STATE;
use crate::session::{Dword, read_all, write_all};
use rc2k_core::menu::{bests, standings};

const SCORE: &Dword = &RESULT_FIGURES.totals[0]; // dword_7587B8
const SCORE_1: &Dword = &RESULT_FIGURES.totals[1]; // dword_7587BC
// hhmm
const CARS_LEN: u32 = 0x28;
const SHOWN_LIST_LEN: u32 = 0x30;
const STAGE_RECORD_LEN: u32 = 0xA4;

use crate::menu::event_flow::dword as get;
/// The stage's record (`off_73555A[level]` + stage x 0xA4).
fn stage_record() -> Result<u32, Refused> {
    let (level, stage) = (ARCADE_GAME.level.get(), ARCADE_GAME.stage.get());
    if level > 5 || stage > 6 {
        return Err(BadInput("an arcade level past 5 or stage past 6").into());
    }
    Ok(crate::records::stages_store::row(
        crate::records::stages_store::STAGE_TABLES.level_table(level)?,
        stage,
    )?)
}

/// `f` on the course's ten best times (`saves::best_times::COURSE_TIMES`); refused with no
/// table or a course past it.
fn with_course_times<R>(
    f: impl FnOnce(&mut rc2k_core::saves::entries::Times) -> R,
) -> Result<R, Refused> {
    let course = get(stage_record()? + 0x3C)? as usize;
    Ok(crate::saves::best_times::COURSE_TIMES
        .with(|t| t.stages.get_mut(course).map(f))?
        .ok_or(BadInput("a course past the best times"))?)
}

/// `f` on the level's ten best scores (`saves::best_times::LEVEL_SCORES`).
fn with_level_scores<R>(
    f: impl FnOnce(&mut rc2k_core::saves::entries::List) -> R,
) -> Result<R, Refused> {
    let level = ARCADE_GAME.level.get();
    if level > 5 {
        return Err(BadInput("an arcade level past 5").into());
    }
    let scores = &crate::saves::best_times::LEVEL_SCORES;
    if !scores.is_taken() {
        return Err(BadInput("no best scores").into());
    }
    Ok(scores.with(|t| f(&mut t.levels[level as usize]))?)
}

fn current() -> Result<(u32, crate::menu::standings::PlayerCell), Refused> {
    let n = EVENT_STATE.current_player.get();
    match crate::menu::player::player(n) {
        None => Err(BadInput("a current player past 3").into()),
        Some(r) => Ok((n, r)),
    }
}

/// `menu::keep_arcade_stage_best`, `record` the caller's edi.
pub fn keep_arcade_stage_best(record: u32) -> Result<(), Refused> {
    if record < 0x10000 {
        return Err(BadInput("an edi that points nowhere").into());
    }
    let (_, player) = current()?;
    // Converted: `rc2k_core::menu::bests::{takes_record, take_record}`.
    let figure = player.get().figure;
    if bests::takes_record(figure, get(record + 0x30)?) {
        let name = player.get().name;
        if name == 0 {
            return Err(BadInput("a null name").into());
        }
        let holder = bests::Holder {
            figure,
            nationality: player.get().nationality,
            car: player.get().car_number,
            name: bytes(name)?,
        };
        let mut r = table(record, STAGE_RECORD_LEN, 1)?;
        bests::take_record(&mut r.entries(), 0, &bests::RECORD_HOLDER, &holder);
    }
    Ok(())
}

pub fn check_arcade_level_score() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::bests::makes_score`.
    let best = with_level_scores(|ten| ten[9].key)?;
    if bests::makes_score(RESULT_FIGURES.totals[0].get(), best) {
        EVENT_STATE.time_qualifies.set(1);
    }
    Ok(())
}

pub fn check_arcade_stage_time() -> Result<(), Refused> {
    with_course_times(|_| ())?;
    let (_, player) = current()?;
    let time = player.get().time;
    EVENT_STATE.player_time.set(time);
    // Converted: `rc2k_core::menu::bests::makes_time`.
    if bests::makes_time(time, with_course_times(|ten| ten[9].head.key)?) {
        EVENT_STATE.time_qualifies.set(1);
    }
    Ok(())
}

/// The player's new entry (player 1): +0x1C, +8, +0xC, +0x10 and the name from its
/// record.
fn new_best(
    player: crate::menu::standings::PlayerCell,
    key: u32,
    seal: Option<bests::Seal>,
    line: Option<[u8; bests::LINE_LEN]>,
) -> Result<bests::NewBest, BadInput> {
    let name_at = player.get().name;
    Ok(bests::NewBest {
        key,
        player: 1,
        nationality: player.get().nationality,
        name_at,
        car: player.get().car_number,
        group: player.get().class,
        name: bytes(name_at)?,
        entered: None,
        seal,
        line,
    })
}

pub fn enter_arcade_stage_time() -> Result<(), Refused> {
    let record = stage_record()?;
    with_course_times(|_| ())?;
    let (_, player) = current()?;
    if player.get().name == 0 {
        return Err(BadInput("a null name").into());
    }
    let course = get(record + 0x3C)?;
    DRAW_WORK.figure.set(course);
    let cheats = CHEAT_VARS.cheats.get();
    let sealed = crate::menu::sealed_player_time::sealed_player_time(course)?;
    // Converted: `rc2k_core::saves::entries` (`BestTime::enter`, `sort_arcade_times`).
    let seal = bests::Seal { cheats, sealed };
    let best = new_best(
        player,
        player.get().time,
        Some(seal),
        Some(player.get().gap_20c),
    )?;
    with_course_times(|ten| {
        ten[9].enter(&best);
        with_kept_name(|kept| rc2k_core::saves::entries::sort_arcade_times(ten, kept));
    })?;
    Ok(())
}

pub fn enter_arcade_level_score() -> Result<(), Refused> {
    with_level_scores(|_| ())?;
    let (n, player) = current()?;
    if player.get().name == 0 {
        return Err(BadInput("a null name").into());
    }
    // Converted: `rc2k_core::saves::entries` (`Entry::enter`, `sort_level_scores`).
    let score = (if n == 1 { SCORE_1 } else { SCORE }).get();
    let best = new_best(player, score, None, None)?;
    with_level_scores(|ten| {
        ten[9].enter(&best);
        with_kept_name(|kept| rc2k_core::saves::entries::sort_level_scores(ten, kept));
    })?;
    Ok(())
}

pub fn score_arcade_stage() -> Result<(), Refused> {
    let first = crate::menu::standings::player(0);
    RESULT_FIGURES.before[0].set(RESULT_FIGURES.totals[0].get());
    RESULT_FIGURES.extra[0].set(first.get().figure);
    // Converted: `rc2k_core::menu::standings::arcade_points`.
    let points =
        standings::arcade_points(ARCADE_GAME.places.get(), RESULT_FIGURES.positions[0].get());
    RESULT_FIGURES.points[0].set(points);
    let sum = points
        .wrapping_add(RESULT_FIGURES.time_bonus[0].get())
        .wrapping_add(RESULT_FIGURES.extra[0].get());
    RESULT_FIGURES.sums[0].set(sum);
    RESULT_FIGURES.totals[0].set(RESULT_FIGURES.before[0].get().wrapping_add(sum));
    Ok(())
}

pub fn clear_arcade_score_marks() -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::entries::clear_marks`.
    with_level_scores(rc2k_core::saves::entries::clear_marks)?;
    Ok(())
}

pub fn clear_arcade_time_marks() -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::entries::clear_marks`.
    with_course_times(rc2k_core::saves::entries::clear_marks)?;
    Ok(())
}

pub fn take_arcade_stage_best() -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::entries::BestTime::line`.
    let line = with_course_times(|ten| ten[0].line())?;
    RACE_STATE.results.results[0].data.write(&line);
    Ok(())
}

/// The place of a list's end mark.
fn end_of(list: &[Dword], len: u32) -> Result<u32, Refused> {
    (0..len)
        .find(|&k| list[k as usize].get() == u32::MAX)
        .ok_or(BadInput("a list of cars without its end").into())
}

pub fn add_shown_car() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::car_lists` (`entries`, `shown_car`).
    use rc2k_core::menu::car_lists::{entries, shown_car};
    let list = |cells: &[Dword], len: u32| entries(len as usize, |k| cells[k].get());
    let shown = EVENT_STATE.shown_car.get();
    let car = shown_car(&list(&MENU_LISTS.shown_cars, SHOWN_LIST_LEN)?, shown)?;
    let end = list(&MENU_LISTS.cars_second, CARS_LEN)?.len();
    MENU_LISTS.cars_second[end].set(car);
    Ok(())
}

/// `menu::remove_shown_car`: the moving down; `then` the list's numbers
/// (`saves::list_car_numbers`).
pub fn remove_shown_car(then: &mut dyn FnMut() -> Result<(), Refused>) -> Result<(), Refused> {
    let shown = EVENT_STATE.shown_car.get();
    let end = end_of(&MENU_LISTS.shown_cars, SHOWN_LIST_LEN)?;
    if shown >= end {
        return Err(BadInput("a shown car past the list").into());
    }
    // Converted: `rc2k_core::menu::standings::remove_at`.
    let mut cars: [u32; SHOWN_LIST_LEN as usize] = read_all(&MENU_LISTS.shown_cars);
    standings::remove_at(&mut cars, shown as usize, end as usize);
    write_all(&MENU_LISTS.shown_cars, cars);
    then()
}
