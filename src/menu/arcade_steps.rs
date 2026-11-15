//! The arcade event's small steps (`menu::arcade_event` and its callers): plain stores on the
//! arcade's level (1 .. 5), stage (0 .. 6), the stage's record (0xA4 bytes; its +0x3C is the
//! course), the course's ten best times (entries of 0x9C), the level's ten best scores (entries
//! of 0x50) and the current player.
//! - keep / take the stage best: the record's best (+0x30) and its setup bytes.
//! - check the level score / stage time: set the time-qualifies flag when the figure beats the
//!   tenth entry's +0x14 (signed).
//! - enter a stage time / level score: the player's figures go into the tenth entry, then the
//!   entries are sorted (an exchange sort from the tenth up; the lower figure moves up, ties
//!   move too; names exchanged through the kept player name).
//! - `menu::score_arcade_stage`: total before, bonus, places' points
//!   ((places ranked - place + 1) x 1000) and their sum with the time bonus.
//! - clear the score / time marks (+0x40 of the ten entries); add / remove the shown car in the
//!   car list (the added car gets no new end mark, kept as is); `secret_car_list`.
//!
//! The entries' logic is `rc2k_core::menu::bests` and `rc2k_core::menu::standings`; this file
//! reads and writes the game's memory for it. Refusals (before anything): a level past 5, a
//! stage past 6, a missing record or table, a current player past 3 or missing, a null name, a
//! list without its end in 0x28 (0x30 for the carousel's list) entries, a shown car past the
//! list.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::owner::Play;
use crate::menu::standings::{name_address_of, name_of, name_player, with_kept_name};
use crate::race::owner::Race;
use crate::records::stages_store::{Holder, RecordRef};
use crate::session::{PlainDword, read_all, write_all};
use crate::system::owner::System;
use rc2k_core::menu::{bests, standings};

const CARS_LEN: u32 = 0x28;
const SHOWN_LIST_LEN: u32 = 0x30;

/// The stage's record: the level's table row for the stage.
fn stage_record(play: &Play, system: &System) -> Result<RecordRef, Refused> {
    let (level, stage) = play.arcade.position();
    if level > 5 || stage > 6 {
        return Err(BadInput("an arcade level past 5 or stage past 6").into());
    }
    let stages = &system.records().stage_tables;
    Ok(crate::records::stages_store::row(
        stages,
        stages.level_table(level)?,
        stage,
    )?)
}

/// `f` on the course's ten best times (`saves::best_times::COURSE_TIMES`); refused with no
/// table or a course past it.
fn with_course_times<R>(
    play: &Play,
    system: &System,
    f: impl FnOnce(&mut rc2k_core::saves::entries::Times) -> R,
) -> Result<R, Refused> {
    let course = stage_record(play, system)?
        .record(&system.records().stage_tables)?
        .event_slot() as usize;
    Ok(crate::saves::best_times::COURSE_TIMES
        .with(|t| t.stages.get_mut(course).map(f))?
        .ok_or(BadInput("a course past the best times"))?)
}

/// `f` on the level's ten best scores (`saves::best_times::LEVEL_SCORES`).
fn with_level_scores<R>(
    play: &Play,
    f: impl FnOnce(&mut rc2k_core::saves::entries::List) -> R,
) -> Result<R, Refused> {
    let level = play.arcade.level.get();
    if level > 5 {
        return Err(BadInput("an arcade level past 5").into());
    }
    let scores = &crate::saves::best_times::LEVEL_SCORES;
    if !scores.is_taken() {
        return Err(BadInput("no best scores").into());
    }
    Ok(scores.with(|t| f(&mut t.levels[level as usize]))?)
}

fn current(menus: &Menus) -> Result<(u32, crate::menu::standings::PlayerCell<'_>), Refused> {
    let n = menus.screens.event.current_player.get();
    match crate::menu::player::player(menus, n) {
        None => Err(BadInput("a current player past 3").into()),
        Some(r) => Ok((n, r)),
    }
}

/// `menu::keep_arcade_stage_best`; `record` is the caller's stage record.
pub fn keep_arcade_stage_best(
    menus: &Menus,
    stages: &crate::records::stages_store::StageTables,
    record: RecordRef,
) -> Result<(), Refused> {
    let (_, player) = current(menus)?;
    let figure = player.get().figure;
    if bests::takes_record(figure, record.record(stages)?.holder(Holder::Fastest).speed) {
        name_player(&player.get())?;
        let (row, name) = crate::menu::standings::as_holder(menus, &player.get())?;
        record
            .record(stages)?
            .set_holder(Holder::Fastest, &row, &name);
    }
    Ok(())
}

pub fn check_arcade_level_score(menus: &Menus) -> Result<(), Refused> {
    let best = with_level_scores(&menus.play, |ten| ten[9].key)?;
    if bests::makes_score(menus.play.results.players[0].total.get(), best) {
        menus.screens.event.trial.qualifies.set(1);
    }
    Ok(())
}

pub fn check_arcade_stage_time(menus: &Menus, system: &System) -> Result<(), Refused> {
    let screens = &menus.screens;
    with_course_times(&menus.play, system, |_| ())?;
    let (_, player) = current(menus)?;
    let time = player.get().time;
    if bests::makes_time(
        time,
        with_course_times(&menus.play, system, |ten| ten[9].head.key)?,
    ) {
        screens.event.trial.qualifies.set(1);
    }
    Ok(())
}

/// The player's new entry (player 1): +0x1C, +8, +0xC, +0x10 and the name from its
/// record.
fn new_best(
    menus: &Menus,
    player: crate::menu::standings::PlayerCell,
    key: u32,
    seal: Option<bests::Seal>,
    line: Option<[u8; bests::LINE_LEN]>,
) -> Result<bests::NewBest, BadInput> {
    let name_at = name_address_of(&player.get())?;
    Ok(bests::NewBest {
        key,
        player: 1,
        nationality: player.get().nationality,
        name_at,
        car: player.get().car_number,
        group: player.get().class,
        name: name_of(menus, &player.get())?,
        entered: None,
        seal,
        line,
    })
}

pub fn enter_arcade_stage_time(
    menus: &Menus,
    system: &System,
    race: &crate::race::owner::Race,
) -> Result<(), Refused> {
    let record = stage_record(&menus.play, system)?;
    with_course_times(&menus.play, system, |_| ())?;
    let (_, player) = current(menus)?;
    name_player(&player.get())?;
    let course = record.record(&system.records().stage_tables)?.event_slot();
    let cheats = menus.play.cheats.cheats.get();
    let sealed = crate::menu::sealed_player_time::sealed_player_time(menus, race, course)?;
    let seal = bests::Seal { cheats, sealed };
    let best = new_best(
        menus,
        player,
        player.get().time,
        Some(seal),
        Some(player.get().best_line),
    )?;
    with_course_times(&menus.play, system, |ten| {
        ten[9].enter(&best);
        with_kept_name(&menus.play, |kept| {
            rc2k_core::saves::entries::sort_arcade_times(ten, kept);
        });
    })?;
    Ok(())
}

pub fn enter_arcade_level_score(menus: &Menus) -> Result<(), Refused> {
    with_level_scores(&menus.play, |_| ())?;
    let (n, player) = current(menus)?;
    name_player(&player.get())?;
    let score = menus.play.results.players[usize::from(n == 1)].total.get();
    let best = new_best(menus, player, score, None, None)?;
    with_level_scores(&menus.play, |ten| {
        ten[9].enter(&best);
        with_kept_name(&menus.play, |kept| {
            rc2k_core::saves::entries::sort_level_scores(ten, kept);
        });
    })?;
    Ok(())
}

pub fn score_arcade_stage(menus: &Menus) -> Result<(), Refused> {
    let play = &menus.play;
    let first = crate::menu::standings::player(menus, 0);
    let player = &play.results.players[0];
    let mut figures = player.get();
    figures.before = figures.total;
    figures.extra = first.get().figure;
    figures.points = standings::arcade_points(play.arcade.places.get(), figures.position);
    figures.sum = figures
        .points
        .wrapping_add(figures.time_bonus)
        .wrapping_add(figures.extra);
    figures.total = figures.before.wrapping_add(figures.sum);
    player.set(figures);
    Ok(())
}

pub fn clear_arcade_score_marks(play: &Play) -> Result<(), Refused> {
    with_level_scores(play, rc2k_core::saves::entries::clear_marks)?;
    Ok(())
}

pub fn clear_arcade_time_marks(play: &Play, system: &System) -> Result<(), Refused> {
    with_course_times(play, system, rc2k_core::saves::entries::clear_marks)?;
    Ok(())
}

pub fn take_arcade_stage_best(play: &Play, system: &System, race: &Race) -> Result<(), Refused> {
    let line = with_course_times(play, system, |ten| ten[0].line())?;
    race.shared.state.results.results[0].data.write(&line);
    Ok(())
}

/// The place of a list's end mark.
fn end_of(list: &[PlainDword], len: u32) -> Result<u32, Refused> {
    (0..len)
        .find(|&k| list[k as usize].get() == u32::MAX)
        .ok_or(BadInput("a list of cars without its end").into())
}

pub fn add_shown_car(menus: &Menus) -> Result<(), Refused> {
    use rc2k_core::menu::car_lists::{entries, shown_car};
    let list = |cells: &[PlainDword], len: u32| entries(len as usize, |k| cells[k].get());
    let shown = menus.screens.event.shown.car.get();
    let car = shown_car(&list(&menus.cars.lists.shown_cars, SHOWN_LIST_LEN)?, shown)?;
    let end = list(&menus.cars.lists.cars_second, CARS_LEN)?.len();
    menus.cars.lists.cars_second[end].set(car);
    Ok(())
}

/// `menu::remove_shown_car`: the moving down; `then` the list's numbers
/// (`saves::list_car_numbers`).
pub fn remove_shown_car(
    menus: &Menus,
    then: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    let shown = menus.screens.event.shown.car.get();
    let end = end_of(&menus.cars.lists.shown_cars, SHOWN_LIST_LEN)?;
    if shown >= end {
        return Err(BadInput("a shown car past the list").into());
    }
    let mut cars: [u32; SHOWN_LIST_LEN as usize] = read_all(&menus.cars.lists.shown_cars);
    standings::remove_at(&mut cars, shown as usize, end as usize);
    write_all(&menus.cars.lists.shown_cars, cars);
    then()
}
