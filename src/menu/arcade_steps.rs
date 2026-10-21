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
//! Refusals (before anything): a level past 5, a stage past 6, a missing record or table
//! on the way, a current player past 3 or missing, a null name, a list without its end
//! in 0x28 (0x30 for `off_75A12E`) entries, a shown car past the list.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::menu::player::{PLAYERS_RVA, player_at};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use rc2k_core::menu::standings;

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
const HHMM_RVA: usize = rva(0x7559F0); // hhmm
const BEST_LINE_RVA: usize = rva(0x75B02C); // unk_75B02C
const CARS_RVA: usize = rva(0x75A042); // off_75A042
const SHOWN_LIST_RVA: usize = rva(0x75A12E); // off_75A12E
const SHOWN_RVA: usize = rva(0x755990); // dword_755990
const SECRET_LIST_RVA: usize = rva(0x75A412); // off_75A412
const CARS_LEN: u32 = 0x28;
const SHOWN_LIST_LEN: u32 = 0x30;
const TIME_LEN: u32 = 0x9C;
const SCORE_LEN: u32 = 0x50;
const NAME: u32 = 31;

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}
fn p(a: usize, v: u32) {
    unsafe { game_write(a, v) }
}
fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}
fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}
unsafe fn copy_bytes(from: u32, to: u32, n: u32) {
    unsafe { std::ptr::copy(from as *const u8, to as *mut u8, n as usize) }
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
    let best = get(player + 0x94);
    if (get(record + 0x30) as i32) < best as i32 {
        let name = get(player + 0x18);
        if name == 0 {
            return Err(BadInput("a null name").into());
        }
        put(record + 0x30, best);
        put(record + 0x74, get(player + 4));
        put(record + 0x78, get(player + 8));
        unsafe { copy_bytes(name, record + 0x54, 0x20) };
    }
    Ok(())
}

pub fn check_arcade_level_score() -> Result<(), Refused> {
    let tenth = level_scores()? + 0x2D0;
    if g(SCORE_RVA) as i32 >= get(tenth + 0x14) as i32 {
        p(FLAG_RVA, 1);
    }
    Ok(())
}

pub fn check_arcade_stage_time() -> Result<(), Refused> {
    let tenth = course_times()? + 0x57C;
    let (_, player) = current()?;
    let time = get(player + 0x88);
    p(TIME_SEEN_RVA, time);
    if time as i32 <= get(tenth + 0x14) as i32 {
        p(FLAG_RVA, 1);
    }
    Ok(())
}

pub fn sort_arcade_stage_times() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::standings::sort_arcade_times`.
    let first = course_times()?;
    let mut ten = unsafe { ten_entries(first, TIME_LEN) };
    standings::sort_arcade_times(&mut ten, unsafe { kept_name() });
    Ok(())
}

pub fn sort_arcade_level_scores() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::standings::sort_arcade_scores`.
    let first = level_scores()?;
    let mut ten = unsafe { ten_entries(first, SCORE_LEN) };
    standings::sort_arcade_scores(&mut ten, unsafe { kept_name() });
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

/// What both entries share: +0x1C, +8, +0xC, +0x10, +4, +0x40, +0x20 and the name.
unsafe fn enter_common(entry: u32, player: u32) {
    put(entry + 0x1C, get(player + 0x90));
    put(entry + 8, get(player + 4));
    put(entry + 0xC, get(player + 0x18));
    put(entry + 0x10, get(player + 8));
}

pub fn enter_arcade_stage_time() -> Result<(), Refused> {
    let record = stage_record()?;
    let tenth = course_times()? + 0x57C;
    let (_, player) = current()?;
    if get(player + 0x18) == 0 {
        return Err(BadInput("a null name").into());
    }
    let course = get(record + 0x3C);
    p(SLOT_RVA, course);
    put(tenth + 0x14, get(player + 0x88));
    unsafe { enter_common(tenth, player) };
    put(tenth + 0x50, g(CHEATS_RVA));
    let sealed = unsafe {
        crate::menu::sealed_player_time::sealed_player_time(&Regs {
            eax: course,
            ..Regs::SENTINEL
        })
    }?;
    put(tenth + 0x54, sealed.eax);
    put(tenth + 0x58, sealed.ebx);
    put(tenth + 4, 1);
    put(tenth + 0x40, 1);
    put(tenth + 0x20, get(player + 0x18));
    unsafe {
        copy_bytes(player + 0x20C, tenth + 0x5C, 0x40);
        copy_bytes(get(tenth + 0xC), tenth + 0x20, NAME);
    }
    sort_arcade_stage_times()
}

pub fn enter_arcade_level_score() -> Result<(), Refused> {
    let tenth = level_scores()? + 0x2D0;
    let (n, player) = current()?;
    if get(player + 0x18) == 0 {
        return Err(BadInput("a null name").into());
    }
    put(
        tenth + 0x14,
        g(if n == 1 { SCORE_1_RVA } else { SCORE_RVA }),
    );
    unsafe { enter_common(tenth, player) };
    put(tenth + 4, 1);
    put(tenth + 0x40, 1);
    put(tenth + 0x20, get(player + 0x18));
    unsafe { copy_bytes(get(tenth + 0xC), tenth + 0x20, NAME) };
    sort_arcade_level_scores()
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
    let first = level_scores()?;
    for k in 0..10 {
        put(first + k * SCORE_LEN + 0x40, 0);
    }
    Ok(())
}

pub fn clear_arcade_time_marks() -> Result<(), Refused> {
    let first = course_times()?;
    for k in 0..10 {
        put(first + k * TIME_LEN + 0x40, 0);
    }
    Ok(())
}

pub fn take_arcade_stage_best() -> Result<(), Refused> {
    let first = course_times()?;
    unsafe { copy_bytes(first + 0x5C, game_addr(BEST_LINE_RVA) as u32, 0x40) };
    Ok(())
}

/// The place of a list's end mark.
fn end_of(list: usize, len: u32) -> Result<u32, Refused> {
    (0..len)
        .find(|&k| g(list + 4 * k as usize) == u32::MAX)
        .ok_or(BadInput("a list of cars without its end").into())
}

pub fn add_shown_car() -> Result<(), Refused> {
    let shown = g(SHOWN_RVA);
    if shown >= end_of(SHOWN_LIST_RVA, SHOWN_LIST_LEN)? {
        return Err(BadInput("a shown car past the list").into());
    }
    let car = g(SHOWN_LIST_RVA + 4 * shown as usize);
    let end = end_of(CARS_RVA, CARS_LEN)?;
    p(CARS_RVA + 4 * end as usize, car);
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
    let list = [(
        game_addr(SHOWN_LIST_RVA) as u32,
        4 * SHOWN_LIST_LEN as usize,
    )];
    let saved = capture(&list);
    // Converted: `rc2k_core::menu::standings::remove_at`.
    let mut cars: [u32; SHOWN_LIST_LEN as usize] = unsafe { game_read(SHOWN_LIST_RVA) };
    standings::remove_at(&mut cars, shown as usize, end as usize);
    unsafe { game_write(SHOWN_LIST_RVA, cars) };
    // A refusal of the numbers' (a list without its end, a null car) leaves nothing written.
    then().inspect_err(|_| restore(&list, &saved))
}

/// `menu::secret_car_list`: what it leaves.
pub fn secret_car_list() -> Left {
    let on = g(CHEATS_RVA) & 2 != 0;
    let left = Left::carry(on);
    if on {
        left.with(crate::framework::ESI, game_addr(SECRET_LIST_RVA) as u32)
    } else {
        left
    }
}

// ---- hooks' and tests' common parts ----

/// What they may write, for the shadow comparison: the variables, the stage's records,
/// the course's times, the level's scores, the current player's record, the car lists.
pub fn regions() -> Vec<Region> {
    let mut r = vec![
        at(FLAG_RVA),
        at(TIME_SEEN_RVA),
        (game_addr(BONUS_RVA) as u32, 0x70), // 758758 .. 7587C8
        (game_addr(KEPT_NAME_RVA) as u32, 0x20),
        at(SLOT_RVA),
        at(HHMM_RVA),
        (game_addr(BEST_LINE_RVA) as u32, 0x40),
        (game_addr(CARS_RVA) as u32, 4 * CARS_LEN as usize),
        (
            game_addr(SHOWN_LIST_RVA) as u32,
            4 * SHOWN_LIST_LEN as usize,
        ),
        (game_addr(rva(0x755258)) as u32, 0x140), // the two lists of numbers
    ];
    r.extend(crate::race::save_seal::seal_regions());
    r.push((game_addr(crate::race::save_seal::BLOCK_RVA) as u32, 8));
    if let Ok(s) = stage_record() {
        r.push((s, 0xA4));
    }
    if let Ok(t) = course_times() {
        r.push((t, 0x618));
    }
    if let Ok(s) = level_scores() {
        r.push((s, 0x320));
    }
    if let Ok((_, p)) = current() {
        r.push((p, std::mem::size_of::<crate::menu::player::Player>()));
    }
    r
}

pub fn dispatch_step(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    rust: crate::service::harness::Rust,
) {
    dispatch_state(hook, r, call_original, &regions(), rust);
}

/// Test memory in the places the steps read: level 2 stage 3 whose course is 1, the
/// times and scores of the test's, two player records with names, cheats; the car lists
/// of the test's in the game's lists' places.
struct Fixture {
    stages: Vec<u8>,
    times: Vec<u8>,
    scores: Vec<u8>,
    players: Vec<u8>,
    names: Vec<u8>,
    inputs: Vec<Region>,
    saved: Vec<u8>,
}

fn inputs() -> Vec<Region> {
    let mut r = regions();
    r.extend([
        at(LEVEL_RVA),
        at(STAGE_RVA),
        (game_addr(STAGE_TABLES_RVA) as u32, 0x18),
        at(TIMES_BASE_RVA),
        at(SCORES_BASE_RVA),
        at(CURRENT_RVA),
        (game_addr(PLAYERS_RVA) as u32, 0x10),
        at(CHEATS_RVA),
        at(SHOWN_RVA),
        at(PLACES_RVA),
        at(rva(0x75ACD8)),
    ]);
    r
}

const PLAYER_LEN: usize = 0x300;

impl Fixture {
    fn new() -> Self {
        let inputs = inputs();
        let saved = capture(&inputs);
        let mut f = Self {
            stages: vec![0; 7 * 0xA4],
            times: vec![0; 2 * 0x618],
            scores: vec![0; 6 * 0x320],
            players: vec![0; 2 * PLAYER_LEN],
            names: vec![0; 0x40],
            inputs,
            saved,
        };
        unsafe {
            p(LEVEL_RVA, 2);
            p(STAGE_RVA, 3);
            p(STAGE_TABLES_RVA + 8, f.stages.as_mut_ptr() as u32);
            p(TIMES_BASE_RVA, f.times.as_mut_ptr() as u32);
            p(SCORES_BASE_RVA, f.scores.as_mut_ptr() as u32);
            p(PLAYERS_RVA, f.players.as_mut_ptr() as u32);
            p(
                PLAYERS_RVA + 4,
                f.players.as_mut_ptr().add(PLAYER_LEN) as u32,
            );
            game_write(rva(0x75ACD8), 2u32);
        }
        f
    }

    /// Contents by `seed`; the times' and scores' +0x14 from `keys` (top to bottom).
    fn fill(&mut self, seed: u32, keys: &[i32; 10], player: u32) {
        let pattern = |v: &mut Vec<u8>, salt: u32| {
            for (k, b) in v.iter_mut().enumerate() {
                *b = ((k as u32).wrapping_mul(0x3B) ^ seed.wrapping_mul(salt)) as u8;
            }
        };
        pattern(&mut self.stages, 7);
        pattern(&mut self.times, 11);
        pattern(&mut self.scores, 13);
        pattern(&mut self.players, 17);
        self.names[..0x20].copy_from_slice(b"Test driver name, thirty-one by\0");
        self.names[0x20..].copy_from_slice(b"Second test name, thirty-one by\0");
        let names = self.names.as_ptr() as u32;
        let stage = self.stages.as_ptr() as u32 + 3 * 0xA4;
        put(stage + 0x3C, 1);
        put(stage + 0x30, keys[0] as u32);
        let times = self.times.as_ptr() as u32 + 0x618;
        let scores = self.scores.as_ptr() as u32 + 2 * 0x320;
        for (k, &key) in keys.iter().enumerate() {
            put(times + k as u32 * TIME_LEN + 0x14, key as u32);
            put(
                scores + k as u32 * SCORE_LEN + 0x14,
                key.wrapping_neg() as u32,
            );
        }
        for n in 0..2u32 {
            let r = self.players.as_ptr() as u32 + n * PLAYER_LEN as u32;
            put(r + 0x18, names + 0x20 * n);
            put(
                r + 0x88,
                keys[(4 + seed as usize) % 10].wrapping_add(n as i32) as u32,
            );
            put(r + 0x94, keys[(seed as usize + n as usize) % 10] as u32);
        }
        p(CURRENT_RVA, player);
        p(SCORE_RVA, keys[seed as usize % 10].wrapping_neg() as u32);
        p(
            SCORE_1_RVA,
            keys[(seed as usize + 3) % 10].wrapping_neg() as u32,
        );
        p(CHEATS_RVA, seed.wrapping_mul(0x9E37_79B9) & 0x7FFFF);
        p(PLACES_RVA, 5 + seed);
        p(POSITION_RVA, seed % 4);
        p(FLAG_RVA, 0);
        // The car lists: shown car 1 of five, a list of three.
        for k in 0..SHOWN_LIST_LEN as usize {
            p(
                SHOWN_LIST_RVA + 4 * k,
                if k < 5 {
                    game_addr(rva(0x757210)) as u32 + 0x78 * k as u32
                } else {
                    u32::MAX
                },
            );
        }
        for k in 0..CARS_LEN as usize {
            p(
                CARS_RVA + 4 * k,
                if k < 3 {
                    game_addr(rva(0x757300)) as u32 + 0x78 * k as u32
                } else {
                    u32::MAX
                },
            );
        }
        p(SHOWN_RVA, seed % 5);
    }

    fn end(self) {
        restore(&self.inputs, &self.saved);
    }
}

/// The self-test of each step: the fixture filled by several seeds and orders of the
/// best lists (sorted, a tie, the tenth best and worst), the current player 0 and 1.
pub fn test_step(
    hook: &RawHook,
    call_original: CallOriginal,
    rust: crate::service::harness::Rust,
    before: &dyn Fn(u32) -> Regs,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let mut f = Fixture::new();
    let orders: [[i32; 10]; 4] = [
        [100, 200, 300, 400, 500, 600, 700, 800, 900, 1000],
        [100, 200, 300, 400, 500, 600, 700, 800, 900, 50],
        [100, 200, 300, 300, 500, 600, 600, 800, 900, 300],
        [-5, 0x7FFF_FFFF, 3, 3, i32::MIN, 0, 1, 2, 3, 3],
    ];
    let fixture = std::cell::RefCell::new(&mut f);
    for (n, keys) in orders.iter().enumerate() {
        for seed in [0u32, 1, 2] {
            for player in [0u32, 1] {
                let stage = fixture.borrow().stages.as_ptr() as u32 + 3 * 0xA4;
                check_state(
                    &mut t,
                    &format!("order {n}, seed {seed}, player {player}"),
                    call_original,
                    before(stage),
                    &|| fixture.borrow_mut().fill(seed, keys, player),
                    &regions(),
                    rust,
                );
            }
        }
    }
    f.end();
    t.finish()
}
