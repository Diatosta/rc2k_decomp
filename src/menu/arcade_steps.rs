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
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::menu::player::player_at;
use crate::menu::standings::{bytes, table};
use rc2k_core::menu::{bests, standings};

pub const LEVEL_RVA: usize = rva(0x7587EC); // dword_7587EC
pub const STAGE_RVA: usize = rva(0x7587F0); // dword_7587F0
const STAGE_TABLES_RVA: usize = rva(0x73555A); // off_73555A
const TIMES_BASE_RVA: usize = rva(0x7556E0); // dword_7556E0
const SCORES_BASE_RVA: usize = rva(0x7556DC); // dword_7556DC
const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const FLAG_RVA: usize = rva(0x755918); // dword_755918
const TIME_SEEN_RVA: usize = rva(0x755914); // dword_755914
const SCORE_RVA: usize = rva(0x7587B8); // dword_7587B8
const SCORE_1_RVA: usize = rva(0x7587BC); // dword_7587BC
const BEFORE_RVA: usize = rva(0x7587A8); // dword_7587A8
const EXTRA_RVA: usize = rva(0x758788); // dword_758788
const PLACES_RVA: usize = rva(0x7587F8); // dword_7587F8
const POSITION_RVA: usize = rva(0x758768); // dword_758768
const POINTS_RVA: usize = rva(0x758778); // dword_758778
const BONUS_RVA: usize = rva(0x758758); // dword_758758
const SUM_RVA: usize = rva(0x758798); // dword_758798
const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const KEPT_NAME_RVA: usize = rva(0x755884); // g_keptPlayerName
const SLOT_RVA: usize = rva(0x7559EC); // dword_7559EC
// hhmm
const BEST_LINE_RVA: usize = rva(0x75B02C); // unk_75B02C
const CARS_RVA: usize = rva(0x75A042); // off_75A042
const SHOWN_LIST_RVA: usize = rva(0x75A12E); // off_75A12E
const SHOWN_RVA: usize = rva(0x755990); // dword_755990
const CARS_LEN: u32 = 0x28;
const SHOWN_LIST_LEN: u32 = 0x30;
const TIME_LEN: u32 = 0x9C;
const SCORE_LEN: u32 = 0x50;
const STAGE_RECORD_LEN: u32 = 0xA4;

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}
fn p(a: usize, v: u32) {
    unsafe { game_write(a, v) }
}
fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}
/// The stage's record (`off_73555A[level]` + stage x 0xA4).
fn stage_record() -> Result<u32, Refused> {
    let (level, stage) = (g(LEVEL_RVA), g(STAGE_RVA));
    if level > 5 || stage > 6 {
        return Err(BadInput("an arcade level past 5 or stage past 6").into());
    }
    match g(STAGE_TABLES_RVA + 4 * level as usize) {
        0 => Err(BadInput("no arcade stage records").into()),
        t => Ok(t + stage * 0xA4),
    }
}

/// The course's ten best times (the first).
fn course_times() -> Result<u32, Refused> {
    let course = get(stage_record()? + 0x3C);
    if course > 0x40 {
        return Err(BadInput("a course past the best times").into());
    }
    match g(TIMES_BASE_RVA) {
        0 => Err(BadInput("no best times").into()),
        b => Ok(b.wrapping_add(course * 0x618)),
    }
}

/// The level's ten best scores (the first).
fn level_scores() -> Result<u32, Refused> {
    let level = g(LEVEL_RVA);
    if level > 5 {
        return Err(BadInput("an arcade level past 5").into());
    }
    match g(SCORES_BASE_RVA) {
        0 => Err(BadInput("no best scores").into()),
        b => Ok(b + level * 0x320),
    }
}

fn current() -> Result<(u32, u32), Refused> {
    let n = g(CURRENT_RVA);
    match player_at(n) {
        0 => Err(BadInput("a current player past 3 or missing").into()),
        r => Ok((n, r)),
    }
}

/// `menu::keep_arcade_stage_best`, `record` the caller's edi.
pub fn keep_arcade_stage_best(record: u32) -> Result<(), Refused> {
    if record < 0x10000 {
        return Err(BadInput("an edi that points nowhere").into());
    }
    let (_, player) = current()?;
    // Converted: `rc2k_core::menu::bests::{takes_record, take_record}`.
    let figure = get(player + 0x94);
    if bests::takes_record(figure, get(record + 0x30)) {
        let name = get(player + 0x18);
        if name == 0 {
            return Err(BadInput("a null name").into());
        }
        let holder = bests::Holder {
            figure,
            nationality: get(player + 4),
            car: get(player + 8),
            name: bytes(name),
        };
        let mut r = unsafe { table(record, STAGE_RECORD_LEN, 1) };
        bests::take_record(&mut r, 0, &bests::RECORD_HOLDER, &holder);
    }
    Ok(())
}

pub fn check_arcade_level_score() -> Result<(), Refused> {
    let tenth = level_scores()? + 0x2D0;
    // Converted: `rc2k_core::menu::bests::makes_score`.
    if bests::makes_score(g(SCORE_RVA), get(tenth + 0x14)) {
        p(FLAG_RVA, 1);
    }
    Ok(())
}

pub fn check_arcade_stage_time() -> Result<(), Refused> {
    let tenth = course_times()? + 0x57C;
    let (_, player) = current()?;
    let time = get(player + 0x88);
    p(TIME_SEEN_RVA, time);
    // Converted: `rc2k_core::menu::bests::makes_time`.
    if bests::makes_time(time, get(tenth + 0x14)) {
        p(FLAG_RVA, 1);
    }
    Ok(())
}

/// The ten entries from `first`, `len` bytes each.
///
/// # Safety
/// They must be game memory no one else uses meanwhile.
unsafe fn ten_entries<'a>(first: u32, len: u32) -> standings::Entries<'a> {
    standings::Entries {
        bytes: unsafe { std::slice::from_raw_parts_mut(first as *mut u8, 10 * len as usize) },
        size: len as usize,
    }
}

/// `g_keptPlayerName`, the sorts' swap buffer.
unsafe fn kept_name<'a>() -> &'a mut [u8; standings::NAME_LEN] {
    unsafe { &mut *(game_addr(KEPT_NAME_RVA) as *mut [u8; standings::NAME_LEN]) }
}

/// The player's new entry (player 1): +0x1C, +8, +0xC, +0x10 and the name from its
/// record.
fn new_best(
    player: u32,
    key: u32,
    seal: Option<bests::Seal>,
    line: Option<[u8; bests::LINE_LEN]>,
) -> bests::NewBest {
    let name_at = get(player + 0x18);
    bests::NewBest {
        key,
        player: 1,
        nationality: get(player + 4),
        name_at,
        car: get(player + 8),
        group: get(player + 0x90),
        name: bytes(name_at),
        entered: None,
        seal,
        line,
    }
}

pub fn enter_arcade_stage_time() -> Result<(), Refused> {
    let record = stage_record()?;
    let first = course_times()?;
    let (_, player) = current()?;
    if get(player + 0x18) == 0 {
        return Err(BadInput("a null name").into());
    }
    let course = get(record + 0x3C);
    p(SLOT_RVA, course);
    let cheats = g(CHEATS_RVA);
    let sealed = unsafe { crate::menu::sealed_player_time::sealed_player_time(course) }?;
    // Converted: `rc2k_core::menu::bests::enter_tenth`, then
    // `rc2k_core::menu::standings::sort_arcade_times`.
    let seal = bests::Seal { cheats, sealed };
    let best = new_best(
        player,
        get(player + 0x88),
        Some(seal),
        Some(bytes(player + 0x20C)),
    );
    let mut ten = unsafe { ten_entries(first, TIME_LEN) };
    bests::enter_tenth(&mut ten, &best);
    standings::sort_arcade_times(&mut ten, unsafe { kept_name() });
    Ok(())
}

pub fn enter_arcade_level_score() -> Result<(), Refused> {
    let first = level_scores()?;
    let (n, player) = current()?;
    if get(player + 0x18) == 0 {
        return Err(BadInput("a null name").into());
    }
    // Converted: `rc2k_core::menu::bests::enter_tenth`, then
    // `rc2k_core::menu::standings::sort_arcade_scores`.
    let score = g(if n == 1 { SCORE_1_RVA } else { SCORE_RVA });
    let best = new_best(player, score, None, None);
    let mut ten = unsafe { ten_entries(first, SCORE_LEN) };
    bests::enter_tenth(&mut ten, &best);
    standings::sort_arcade_scores(&mut ten, unsafe { kept_name() });
    Ok(())
}

pub fn score_arcade_stage() -> Result<(), Refused> {
    let first = match player_at(0) {
        0 => return Err(BadInput("no player 0").into()),
        r => r,
    };
    p(BEFORE_RVA, g(SCORE_RVA));
    p(EXTRA_RVA, get(first + 0x94));
    // Converted: `rc2k_core::menu::standings::arcade_points`.
    let points = standings::arcade_points(g(PLACES_RVA), g(POSITION_RVA));
    p(POINTS_RVA, points);
    let sum = points.wrapping_add(g(BONUS_RVA)).wrapping_add(g(EXTRA_RVA));
    p(SUM_RVA, sum);
    p(SCORE_RVA, g(BEFORE_RVA).wrapping_add(sum));
    Ok(())
}

pub fn clear_arcade_score_marks() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::bests::clear_marks`.
    let first = level_scores()?;
    bests::clear_marks(&mut unsafe { ten_entries(first, SCORE_LEN) });
    Ok(())
}

pub fn clear_arcade_time_marks() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::bests::clear_marks`.
    let first = course_times()?;
    bests::clear_marks(&mut unsafe { ten_entries(first, TIME_LEN) });
    Ok(())
}

pub fn take_arcade_stage_best() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::bests::best_line`.
    let first = course_times()?;
    let line = bests::best_line(&unsafe { ten_entries(first, TIME_LEN) });
    unsafe { game_write(BEST_LINE_RVA, line) };
    Ok(())
}

/// The place of a list's end mark.
fn end_of(list: usize, len: u32) -> Result<u32, Refused> {
    (0..len)
        .find(|&k| g(list + 4 * k as usize) == u32::MAX)
        .ok_or(BadInput("a list of cars without its end").into())
}

pub fn add_shown_car() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::car_lists` (`entries`, `shown_car`).
    use rc2k_core::menu::car_lists::{entries, shown_car};
    let list = |at: usize, len: u32| entries(len as usize, |k| g(at + 4 * k));
    let shown = g(SHOWN_RVA);
    let car = shown_car(&list(SHOWN_LIST_RVA, SHOWN_LIST_LEN)?, shown)?;
    let end = list(CARS_RVA, CARS_LEN)?.len();
    p(CARS_RVA + 4 * end, car);
    Ok(())
}

/// `menu::remove_shown_car`: the moving down; `then` the list's numbers
/// (`saves::list_car_numbers`).
pub fn remove_shown_car(then: &mut dyn FnMut() -> Result<(), Refused>) -> Result<(), Refused> {
    let shown = g(SHOWN_RVA);
    let end = end_of(SHOWN_LIST_RVA, SHOWN_LIST_LEN)?;
    if shown >= end {
        return Err(BadInput("a shown car past the list").into());
    }
    // Converted: `rc2k_core::menu::standings::remove_at`.
    let mut cars: [u32; SHOWN_LIST_LEN as usize] = unsafe { game_read(SHOWN_LIST_RVA) };
    standings::remove_at(&mut cars, shown as usize, end as usize);
    unsafe { game_write(SHOWN_LIST_RVA, cars) };
    then()
}

// ---- hooks' and tests' common parts ----
