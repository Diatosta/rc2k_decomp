//! Race init's "cars" (`race::set_up_cars`): by the kind of race one of `race::set_up_mode_1_cars`
//! / `race::set_up_mode_4_cars` / `race::set_up_race_cars` / `race::set_up_mode_0_cars` (each: the
//! car blocks reset, a two-player rule, the two players' kinds) and the cars' data file read
//! (`race::read_stage_car_numbers`: per stage, three-digit numbers into `stage_numbers`).

use crate::effects::cell_x87::CellX87;
use crate::errors::{BadInput, Refused};
use crate::math::x87_float as bh;
use crate::memory::session::{PlainDword, PlainFloat};
use crate::physics::car_store::Car;
#[cfg(test)]
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::race::tables_store::Listed;

/// The eight car blocks (fixed; `car_blocks` lists them), 0xFE0 apart. Player n's (0, 1) kind, tyre
/// and colours word: `car_kinds[n]`, `car_tyres[n]`, `car_colours[n]`.
fn player_kinds(shared: &RaceShared, n: usize) -> [&dyn crate::memory::session::DwordLike; 3] {
    let v = shared.vars.get();
    [&v.race_car_kinds[n], &v.car_tyres[n], &v.car_colours[n]]
}
/// The text the digits are read from (`car_data_text`, 0x708 bytes).
pub const TEXT_LEN: usize = 0x708;

/// A selector of one 32-bit field of a car block: a whole number or a float.
#[derive(Clone, Copy)]
enum Field {
    Word(fn(&Car) -> &PlainDword),
    Float(fn(&Car) -> &PlainFloat),
}

impl Field {
    /// Stores `v` into the field of `car` (a float field takes the float whose bits `v` is).
    fn set(self, car: &Car, v: u32) {
        match self {
            Field::Word(field) => field(car).set(v),
            Field::Float(field) => field(car).set(f32::from_bits(v)),
        }
    }
}

/// The block fields `reset_block` sets to 0, in order.
const CLEARED: [Field; 22] = [
    Field::Word(|c| &c.view_covered),
    Field::Word(|c| &c.splits_recorded),
    Field::Word(|c| &c.stage_time),
    Field::Word(|c| &c.finish_flag),
    Field::Word(|c| &c.go_stand_timer),
    Field::Word(|c| &c.started_stand_timer),
    Field::Word(|c| &c.damage_failed),
    Field::Word(|c| &c.checkpoint),
    Field::Float(|c| &c.brake_heat[0]),
    Field::Float(|c| &c.brake_heat[1]),
    Field::Float(|c| &c.damper_wear[0]),
    Field::Float(|c| &c.damper_wear[1]),
    Field::Float(|c| &c.engine_heat),
    Field::Float(|c| &c.brake_fade),
    Field::Word(|c| &c.handbrake_failed),
    Field::Word(|c| &c.electrical_failures[0]),
    Field::Word(|c| &c.electrical_failures[1]),
    Field::Word(|c| &c.electrical_failures[2]),
    Field::Word(|c| &c.electrical_failures[3]),
    Field::Word(|c| &c.electrical_failures[4]),
    Field::Word(|c| &c.electrical_failures[5]),
    Field::Word(|c| &c.electrical_failures[6]),
];

/// What `race::reset_car_blocks` writes into car block `n` (field, value, in order): `driver_kind`
/// 1 for the race's cars (`cars`, unsigned) else 0, the `CLEARED` fields 0, `gearbox_kind` 1,
/// `time_left` the stage's number (`stage_numbers`) x 100.
fn reset_block(n: u32, cars: u32, number: u32) -> Vec<(Field, u32)> {
    let mut writes: Vec<(Field, u32)> = vec![(Field::Word(|c| &c.driver_kind), (n < cars) as u32)];
    writes.extend(CLEARED.iter().map(|&field| (field, 0)));
    writes.push((Field::Word(|c| &c.gearbox_kind), 1));
    writes.push((Field::Word(|c| &c.time_left), number.wrapping_mul(100)));
    writes
}

/// Resets each of the eight car blocks (by `car_blocks`): `driver_kind` 1 for the race's cars
/// (`race_cars`, unsigned) else 0, `gearbox_kind` 1, `time_left` `stage_numbers` x 100, the others
/// in `CLEARED` 0; then each block's `car_kind` its kind from `race_car_kinds`.
pub fn reset_car_blocks(race: &Race) -> Result<(), BadInput> {
    let physics = &race.physics;
    let shared = &race.shared;
    use rc2k_core::race::cars::CAR_BLOCKS as BLOCK_COUNT;
    {
        let cars: u32 = shared.view.get().race_cars.get();
        for n in 0..BLOCK_COUNT {
            let car = shared.race_car(n as usize)?;
            for (field, v) in reset_block(n, cars, physics.split_tables().stage_numbers[0].get()) {
                field.set(car, v);
            }
        }
        for n in 0..BLOCK_COUNT as usize {
            race.car_blocks().cars[n]
                .car_kind
                .set(shared.vars.get().race_car_kinds[n].get());
        }
    }
    Ok(())
}

/// Unless `run_kind` is 2 or 4, with `players` 2 (two players): the first player whose block's
/// `stop_state` is not 0 (the first, else the second) is out: `driver_kind` 0, `view_covered` 1.
pub fn drop_absent_player(race: &Race) {
    let shared = &race.shared;
    let blocks = race.car_blocks();
    // Logic in `rc2k_core::race::cars::absent_player`.
    {
        let out = rc2k_core::race::cars::absent_player(
            shared.vars.get().run_kind.get(),
            || shared.view.get().players.get(),
            |n| blocks.cars[n as usize].stop_state.get() != 0,
        );
        if let Some(n) = out {
            blocks.cars[n as usize].driver_kind.set(0u32);
            blocks.cars[n as usize].view_covered.set(1u32);
        }
    }
}

/// The players' slots the race sets up: the two first players' always, and players 3 and 4's when
/// the split screen has them.
pub fn seats(race: &Race) -> usize {
    (race.shared.view.get().players.get() as usize).clamp(2, 4)
}

/// Sets the players' blocks' `car_kind`, `tyre_kind` and `gearbox_kind`
/// from the players' chosen kinds.
pub fn set_player_car_kinds(race: &Race) {
    let shared = &race.shared;
    for n in 0..seats(race) {
        let car = &race.car_blocks().cars[n];
        let to = [&car.car_kind, &car.tyre_kind, &car.gearbox_kind];
        for (to, from) in to.into_iter().zip(player_kinds(shared, n)) {
            to.set(from.get());
        }
    }
}

/// The cars' data text from its start (read on past the buffer's end too).
fn text_data(shared: &RaceShared) -> crate::memory::cells::StoreBytes<'_> {
    crate::memory::cells::StoreBytes(&shared.tables().car_data_text)
}

/// Runs `reset_car_blocks`, `drop_absent_player` and `set_player_car_kinds`.
pub fn set_up_race_cars(race: &Race) -> Result<(), BadInput> {
    reset_car_blocks(race)?;
    {
        drop_absent_player(race);
        set_player_car_kinds(race);
    }
    Ok(())
}

// ---- the set-ups ----

/// The most numbers `stage_car_numbers` holds (`split_colour_count` follows it).
pub const MOST_NUMBERS: usize = rc2k_core::race::stage::MOST_NUMBERS;
/// "autosetup", "t", "off", "h", "(0-255)", "s", "(0-255)", "g", "(0-4)", "g", "(0-5)".
const SET_UP_WORDS: [&std::ffi::CStr; 11] = [
    c"autosetup",
    c"t",
    c"off",
    c"h",
    c"(0-255)",
    c"s",
    c"(0-255)",
    c"g",
    c"(0-4)",
    c"g",
    c"(0-5)",
];

/// Log helpers: a text (with or without a line break), a number and a line break.
fn text(text: &std::ffi::CStr, newline: bool) {
    crate::logfile::log_write::log_nul(text.to_bytes(), newline, 1);
}

fn number(value: u32, newline: bool) {
    crate::logfile::log_int::log_int(value as i32, newline, 1);
}

fn newline() {
    text(c"", true);
}

fn word(n: usize) -> &'static std::ffi::CStr {
    SET_UP_WORDS[n]
}

/// Reads the stage's car numbers: sets `stage_car_total` 0 and `read_bytes` 0x708; in
/// `car_data_text` each 'l' or 'L' followed by two digits (0 for none) is a stage's line; on
/// `stage_number`'s, from three places on, three-digit numbers four apart go into
/// `stage_car_numbers` and are summed into `stage_car_total`, up to the text's end or a non-digit.
/// Refuses more than 0x100 numbers (worked out before anything is written).
pub fn read_stage_car_numbers(race: &Race) -> Result<(), Refused> {
    let physics = &race.physics;
    // Logic in `rc2k_core::race::stage::stage_car_numbers`.
    let len = TEXT_LEN as u32;
    let found = rc2k_core::race::stage::stage_car_numbers(
        &text_data(&race.shared),
        len,
        race.shared.vars.get().stage_number.get(),
    )?;
    let mut total = 0u32;
    for (k, &n) in found.numbers.iter().enumerate() {
        physics.split_tables().stage_numbers[k].set(n);
        total = total.wrapping_add(n);
    }
    physics.split_tables().stage_car_total.set(total);
    Ok(())
}

/// Sets up car block `b`: `brake_balance`, `steering_sensitivity`, `ride_height` and
/// `suspension_stiffness` 0x80, `tyre_kind` -1, `gear_ratio_setting` 2; logs "autosetup", "t" and
/// `stage_number`; for stages 0x15 .. 0x4F takes the set-up (stage - 0x15) / 10 x 6 +
/// (stage - 0x15) % 10 of `auto_setups`: `ride_height` its h, `suspension_stiffness` its s,
/// `gear_ratio_setting` its +0xC less one, `tyre_kind` its +8, each logged with its range, then a
/// line break. Refuses stage 0x14, before anything.
pub fn auto_set_up_car(race: &Race, car: &Car) -> Result<(), Refused> {
    let course = &race.course;
    {
        // Logic in `rc2k_core::race::cars::set_up_pick`.
        let stage: u32 = race.shared.vars.get().stage_number.get();
        let pick = rc2k_core::race::cars::set_up_pick(stage)?;
        car.brake_balance.set(0x80);
        car.steering_sensitivity.set(0x80);
        car.ride_height.set(0x80);
        car.suspension_stiffness.set(0x80);
        car.tyre_kind.set(u32::MAX);
        car.gear_ratio_setting.set(2);
        text(word(0), false);
        text(word(1), false);
        number(stage, false);
        let Some(pick) = pick else {
            return Ok(());
        };
        text(word(2), false);
        number(pick.index, false);
        // The pick's set-up: four dwords (h, s, g5, g4 + 1) of the 36, 0x10 bytes each.
        let set_up = |k: usize| {
            (pick.index as usize)
                .checked_mul(4)
                .and_then(|first| course.league().auto_setups.get(first + k))
                .map(crate::memory::session::PlainDword::get)
                .ok_or(BadInput("the auto set-ups"))
        };
        for (value, field, words) in [
            (set_up(0)?, &car.ride_height, [3, 4]),
            (set_up(1)?, &car.suspension_stiffness, [5, 6]),
            (set_up(3)?.wrapping_sub(1), &car.gear_ratio_setting, [7, 8]),
            (set_up(2)?, &car.tyre_kind, [9, 10]),
        ] {
            field.set(value);
            text(word(words[0]), false);
            number(value, false);
            text(word(words[1]), false);
        }
        newline();
    }
    Ok(())
}

/// `RaceState.settings.cars[0].steering_sensitivity` /
/// `RaceState.settings.cars[1].steering_sensitivity` held for a block's `steering_sensitivity`:
/// below 0 is 0, 100 or more 0x100, else times 2.56.
fn brake_value(race: &Race, n: usize) -> u32 {
    let course = &race.course;
    // Logic in `rc2k_core::race::cars::held_brake` (the product the x87's).
    rc2k_core::race::cars::held_brake(
        race.shared.state.car(n).steering_sensitivity.get() as i32,
        |v| crate::math::x87::int_times_to_int(v, &course.numbers.get().brake_scale) as u32,
    )
}

/// Runs `reset_car_blocks`; without a network game, `drop_absent_player`, `set_player_car_kinds` and
/// `auto_set_up_car` of the two players, their `steering_sensitivity` from
/// `RaceState.settings.cars[0].steering_sensitivity` /
/// `RaceState.settings.cars[1].steering_sensitivity` (`brake_value`); else `auto_set_up_car` of
/// every car of the race, then `set_player_car_kinds`. Refuses, in a network game, no cars or more
/// than 8, before anything.
pub fn set_up_mode_0_cars(
    race: &Race,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<(), Refused> {
    {
        // Logic in `rc2k_core::race::cars::{every_car_set_up, held_brake}`.
        let cars: u32 = race.shared.view.get().race_cars.get();
        let all = rc2k_core::race::cars::every_car_set_up(net_race.is_on(), cars)?;
        reset_car_blocks(race)?;
        if !all {
            drop_absent_player(race);
            set_player_car_kinds(race);
            for n in 0..seats(race) {
                auto_set_up_car(race, &race.car_blocks().cars[n])?;
            }
            for n in 0..seats(race) {
                race.car_blocks().cars[n]
                    .steering_sensitivity
                    .set(brake_value(race, n));
            }
        } else {
            for n in 0..cars {
                auto_set_up_car(race, race.shared.race_car(n as usize)?)?;
            }
            set_player_car_kinds(race);
        }
    }
    Ok(())
}

/// Runs `reset_car_blocks`, `drop_absent_player` and `set_player_car_kinds`; sets the two players'
/// `stage_time` from `RaceState.results.service_spent_current` /
/// `RaceState.results.service_spent_first`.
pub fn set_up_mode_1_cars(race: &Race) -> Result<(), BadInput> {
    let shared = &race.shared;
    {
        reset_car_blocks(race)?;
        drop_absent_player(race);
        set_player_car_kinds(race);
        for (n, from) in [
            &shared.state.results.service_spent_current,
            &shared.state.results.service_spent_first,
        ]
        .into_iter()
        .enumerate()
        {
            race.car_blocks().cars[n].stage_time.set(from.get());
        }
    }
    Ok(())
}

/// Sets `race_cars`, `players` and `views` to 1, then runs `reset_car_blocks`,
/// `drop_absent_player` and `set_player_car_kinds`.
pub fn set_up_mode_4_cars(race: &Race) -> Result<(), BadInput> {
    let shared = &race.shared;
    {
        shared.view.get().race_cars.set(1u32);
        shared.view.get().players.set(1u32);
        shared.view.get().views.set(1u32);
        reset_car_blocks(race)?;
        drop_absent_player(race);
        set_player_car_kinds(race);
    }
    Ok(())
}

// ---- the times, mode 2, the whole step ----

/// Zeroes entries 9 .. 0 of the thirteen tables (four of them overlap, one dword apart: the
/// points' tables from entries 0 .. 3).
pub fn clear_race_tables(shared: &RaceShared) {
    let w = shared.work.get();
    let tables: [(&[PlainDword], usize); 13] = [
        (&w.driver_bend_to, 0),
        (&w.driver_bend_noted, 0),
        (&w.driver_steps, 0),
        (&w.driver_reverse_left, 0),
        (&w.driver_bend_turn, 0),
        (&w.driver_points_now, 0),
        (&w.driver_points_now, 1),
        (&w.driver_points_now, 2),
        (&w.driver_points_now, 3),
        (&w.driver_points_then, 0),
        (&w.driver_points_then, 1),
        (&w.driver_points_then, 2),
        (&w.driver_points_then, 3),
    ];
    for k in (0..10).rev() {
        for (table, first) in tables {
            table[first + k].set(0);
        }
    }
}

/// "numcheckposits =", "difficultyindex=", "time percentage=", "total time =", "extra time =",
/// "chkp", "orig time", "car time".
const TIME_WORDS: [&std::ffi::CStr; 8] = [
    c"numcheckposits =",
    c"difficultyindex=",
    c"time percentage=",
    c"total time =",
    c"extra time =",
    c"chkp",
    c"orig time",
    c"car time",
];

/// Sets the car times: `kind_factor` is the factor of the first car's kind (held to 0 .. 0x16);
/// for stages 0x15 .. 0x4E the difficulty (the stage's place in its rally of ten, plus 6 x the
/// rally's level), its time percentage (`time_percentages`) and the extra time (percentage x
/// `stage_car_total` / 100) go into `extra_time` and are added to the first number; each of the
/// `split_colour_count` numbers is multiplied by the factor; all is logged. Stages below 0x15 go
/// the first way too, their index wrapping below the table (a read of the tables before it).
/// Refuses, with a stage, a count of 0 or more than 0x100, before anything.
pub fn set_car_times(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let w = |n: usize| TIME_WORDS[n];
    {
        let stage: u32 = race.shared.vars.get().stage_number.get();
        let count: u32 = race.physics.split_tables().split_colour_count.get();
        if stage < rc2k_core::race::stage::UNTIMED_STAGES
            && (count == 0 || count as usize > MOST_NUMBERS)
        {
            return Err(BadInput("a checkpoint count of 0 or past the numbers").into());
        }
        // Logic in `rc2k_core::race::stage::{timed_kind, rally_of, difficulty}`; the factor's
        // float steps stay the x87's (whole numbers rounded from them).
        use rc2k_core::race::stage::{difficulty, rally_of, timed_kind};
        let kind = timed_kind(race.shared.vars.get().race_car_kinds[0].get() as i32);
        let factor =
            crate::math::x87::loaded(&race.shared.tables().kind_time_factors[kind as usize]);
        race.physics.split_tables().kind_factor.set(factor);
        let Some(rally) = rally_of(stage) else {
            return Ok(());
        };
        let level = difficulty(stage, rally);
        text(w(0), false);
        number(count, false);
        text(w(1), false);
        number(level, false);
        // The index is a stage's place in its rally plus 6 per level of the rally: at most 37
        // (`rally_of`: the stages 0x2F and 0x30 of the last-but-one rally give 36 and 37), and a
        // stage below the first rally's wraps far past the table. The table holds 36, and the two
        // after it read the first two kind time factors' bits.
        let tables = race.shared.tables();
        let percentage = match level.checked_sub(36) {
            None => tables
                .time_percentages
                .get(level as usize)
                .map(crate::memory::session::PlainDword::get),
            Some(k) => tables
                .kind_time_factors
                .get(k as usize)
                .map(crate::memory::session::PlainFloat::bits),
        };
        let percentage = percentage.filter(|_| level <= 37).ok_or(BadInput(
            "a difficulty index past the time percentages and the factors after them",
        ))?;
        text(w(2), false);
        number(percentage, false);
        scene.drawing.scene.get().scratch.set(percentage);
        let total: i32 = race.physics.split_tables().stage_car_total.get() as i32;
        let extra = bh::to_int(
            bh::int(percentage as i32) * bh::int(total) / race.course.numbers.get().hundred.ld(),
        );
        race.physics.split_tables().extra_time.set(extra as u32);
        text(w(3), false);
        number(total as u32, false);
        text(w(4), false);
        number(extra as u32, true);
        race.physics.split_tables().stage_numbers[0].set(
            race.physics.split_tables().stage_numbers[0]
                .get()
                .wrapping_add(extra as u32),
        );
        let mut k = 0u32;
        loop {
            let at = &race.physics.split_tables().stage_numbers[k as usize];
            text(w(5), false);
            number(k, false);
            text(w(6), false);
            number(at.get(), false);
            let n = crate::math::x87::int_times_to_int(at.get() as i32, factor);
            at.set(n as u32);
            text(w(7), false);
            number(n as u32, true);
            k += 1;
            if k == count {
                break;
            }
        }
    }
    Ok(())
}

/// The room `arcade_car_kinds` has (`number_before_first` follows it).
pub const MOST_ARCADE_CARS: u32 = 89;
/// "ARCADE GAME MODE INIT", "rallynum", "stagenum", "NUM arcadecars", "bestpos", "arc start pos
/// (1-n)", "numcarsbehind", " - cartypes ".
const ARCADE_WORDS: [&std::ffi::CStr; 8] = [
    c"ARCADE GAME MODE INIT",
    c"rallynum",
    c"stagenum",
    c"NUM arcadecars",
    c"bestpos",
    c"arc start pos (1-n)",
    c"numcarsbehind",
    c" - cartypes ",
];

/// Sets up the arcade race. From the rally's tables it logs the cars' count (`arcade_cars`),
/// `start_share_last`, `start_share_first`, the start place (the count with `arcade_kind` 2 or the
/// rally's first stage) and the best place. With one player and four cars it runs
/// `reset_car_blocks` and `drop_absent_player`, sets cars 1 .. 3 `steering_sensitivity` 0x80 and
/// `driver_kind` 0, runs `set_player_car_kinds`; from the noise seed `noise_seed` gives each car a
/// type 1 .. 3 (never twice running) and a number (skipping the players' `start_slot_value`), then
/// shuffles the numbers; runs `clear_race_tables` and four `auto_set_up_car`s; sets the player's
/// `steering_sensitivity` (`brake_value`), `pace_placed`, `pace_cars` and the next dword 0 and
/// `mode2_behind` the cars behind; logs the types. Refuses a count of 0 or more than 89 (the
/// tables would be read past), before anything.
pub fn set_up_mode_2_cars(race: &Race) -> Result<(), Refused> {
    let w = |n: usize| ARCADE_WORDS[n];
    let rally: u32 = race.shared.vars.get().race_level.get();
    // The arcade tables hold six by rally (`arcade_cars_by_rally` ..) or stage
    // (`arcade_best_place_by_stage`), back to back: an index past them reads on into the next.
    let pick = |table: Listed, n: u32| race.shared.tables().dword_from(table, n as usize);
    let by_rally = |table: Listed| pick(table, rally);
    let cars: u32 = by_rally(Listed::ArcadeCarsByRally)?;
    if cars == 0 || cars > MOST_ARCADE_CARS {
        return Err(BadInput("an arcade car count of 0 or past the tables").into());
    }
    text(w(0), false);
    text(w(1), false);
    number(rally, false);
    text(w(2), false);
    let stage_in: u32 = race.shared.vars.get().race_stage.get();
    number(stage_in, false);
    race.shared.tables().arcade_cars.set(cars);
    race.shared
        .tables()
        .start_share_last
        .set(f32::from_bits(by_rally(Listed::StartShareLastByRally)?));
    race.shared
        .tables()
        .start_share_first
        .set(f32::from_bits(by_rally(Listed::StartShareFirstByRally)?));
    // Logic in `rc2k_core::race::cars::{arcade_start_is_last, arcade_number, arcade_type_draw,
    // arcade_swap_draw}`.
    use crate::scene::next_noise::next_noise;
    use rc2k_core::race::cars::{
        arcade_number, arcade_start_is_last, arcade_swap_draw, arcade_type_draw,
    };
    if arcade_start_is_last(race.physics.split_tables().arcade_kind.get(), stage_in) {
        race.shared.tables().start_place.set(cars);
    }
    race.shared
        .tables()
        .best_place
        .set(pick(Listed::BestPlaceByStage, stage_in)?);
    text(w(3), false);
    number(cars, false);
    text(w(4), false);
    number(race.shared.tables().best_place.get(), false);
    text(w(5), false);
    number(race.shared.tables().start_place.get(), true);
    race.shared.view.get().views.set(1u32);
    race.shared.view.get().players.set(1u32);
    race.shared.view.get().race_cars.set(4u32);
    reset_car_blocks(race)?;
    drop_absent_player(race);
    for n in 1..4 {
        race.car_blocks().cars[n as usize]
            .steering_sensitivity
            .set(0x80u32);
    }
    for n in 1..4 {
        race.car_blocks().cars[n as usize].driver_kind.set(0u32);
    }
    set_player_car_kinds(race);
    race.shared
        .physics_tables
        .get()
        .noise_state
        .set(race.shared.tables().noise_seed.get());
    // Each car a type (never the last one's again) and a number.
    let mut last_type = u32::MAX;
    let mut number_next = 1u32;
    for car in 0..cars {
        let kind = loop {
            let first: u32 = race.car_blocks().cars[0].start_slot_value.get();
            let second = (race.shared.view.get().players.get() != 1)
                .then(|| race.car_blocks().cars[1].start_slot_value.get());
            number_next = arcade_number(number_next, first, second);
            let (_, kind) = arcade_type_draw(next_noise(&race.shared), car);
            if kind != last_type {
                break kind;
            }
        };
        last_type = kind;
        race.shared.tables().arcade_car_kinds[car as usize].set(kind + 1);
        race.shared.tables().arcade_car_numbers[car as usize].set(number_next);
        number_next += 1;
    }
    // The numbers shuffled.
    for car in 0..cars {
        let mine: u32 = race.shared.tables().arcade_car_numbers[car as usize].get();
        let (_, other_car) = arcade_swap_draw(next_noise(&race.shared), cars);
        let other: u32 = race.shared.tables().arcade_car_numbers[other_car as usize].get();
        race.shared.tables().arcade_car_numbers[car as usize].set(other);
        race.shared.tables().arcade_car_numbers[other_car as usize].set(mine);
    }
    clear_race_tables(&race.shared);
    for n in 0..4 {
        auto_set_up_car(race, &race.car_blocks().cars[n as usize])?;
    }
    race.car_blocks().cars[0]
        .steering_sensitivity
        .set(brake_value(race, 0));
    let lead = &race.car_blocks().cars[0];
    for cell in [&lead.pace_placed, &lead.pace_cars[0], &lead.pace_cars[1]] {
        cell.set(0);
    }
    let behind = cars.wrapping_sub(race.shared.tables().start_place.get());
    race.car_blocks().cars[0].mode2_behind.set(behind);
    text(w(6), false);
    number(behind, false);
    text(w(7), false);
    for car in 0..cars {
        number(
            race.shared.tables().arcade_car_kinds[car as usize].get(),
            false,
        );
    }
    newline();
    Ok(())
}

/// What `race::set_up_cars` writes into both players' blocks first (field, value).
const PLAYER_RESETS: [(Field, u32); 6] = [
    (Field::Float(|c| &c.top_speed), 0),
    (Field::Word(|c| &c.split_gaps[0]), u32::MAX),
    (Field::Word(|c| &c.split_gaps[2]), u32::MAX),
    (Field::Word(|c| &c.split_gaps[1]), u32::MAX),
    (Field::Word(|c| &c.split_gaps[4]), u32::MAX),
    (Field::Word(|c| &c.split_gaps[3]), u32::MAX),
];

/// Race init's "cars": logs "gamemode" and `run_kind`; sets both players' `top_speed` to 0 and
/// `split_gaps` to -1; then by the mode: 2 the numbers, the times, the arcade set-up and
/// `kept_times` (0x40 dwords) 0; 1, 3, 0 (with the numbers first), 4 their set-ups; others
/// nothing.
pub fn set_up_cars(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<(), Refused> {
    text(c"gamemode", false);
    number(race.shared.vars.get().run_kind.get(), true);
    // Logic in `rc2k_core::race::cars::cars_steps`.
    use rc2k_core::race::cars::{CarsStep, cars_steps};
    for n in 0..2 {
        for (field, v) in PLAYER_RESETS {
            field.set(race.shared.race_car(n as usize)?, v);
        }
    }
    for &step in cars_steps(race.shared.vars.get().run_kind.get()) {
        match step {
            CarsStep::Numbers => read_stage_car_numbers(race)?,
            CarsStep::Times => set_car_times(race, scene)?,
            CarsStep::SetUp(0) => set_up_mode_0_cars(race, net_race)?,
            CarsStep::SetUp(1) => set_up_mode_1_cars(race)?,
            CarsStep::SetUp(2) => set_up_mode_2_cars(race)?,
            CarsStep::SetUp(3) => set_up_race_cars(race)?,
            CarsStep::SetUp(_) => set_up_mode_4_cars(race)?,
            CarsStep::ClearArcadeTable => {
                for k in 0..0x40 {
                    race.physics.split_tables().kept_times[k].set(0);
                }
            }
        }
    }
    Ok(())
}

// ---- the arcade's opponents ----

const SEED_WORD: &std::ffi::CStr = c"seed";
/// "p1", "power", "p2", "power", "p3", "power", "p4", "power".
const OPPONENT_WORDS: [&std::ffi::CStr; 8] = [
    c"p1", c"power", c"p2", c"power", c"p3", c"power", c"p4", c"power",
];
const SETUP_ARCADE_WORD: &std::ffi::CStr = c"SETUPARCADECARS";

/// The kinds a draw (0 .. 0x21) can give that `choose_arcade_opponents` takes: usable
/// (`kind_unusable` clear) and within 5 places (`index_of_kind`) of the first car's.
pub fn opponent_kinds(shared: &RaceShared, first: u32) -> Result<Vec<u32>, BadInput> {
    use crate::race::race_shell::{index_of_kind, kind_unusable};
    {
        let first_place = index_of_kind(shared, first)?;
        let mut kinds = Vec::new();
        for k in 0..0x22u32 {
            if !kind_unusable(k).0
                && rc2k_core::race::cars::opponent_distance(index_of_kind(shared, k)?, first_place)
                    .1
                && k != first
            {
                kinds.push(k);
            }
        }
        Ok(kinds)
    }
}

/// With `run_kind` 2, logs the first car's kind's place (`index_of_kind`) ("seed", "p1", "power");
/// then, from the noise seed (`noise_seed`, then `math::next_random`), draws kinds (x 0x22 >> 32)
/// until one is usable, within 5 places of the first's and not taken: the second, third and fourth
/// cars' (`RaceVars.race_car_kinds[1]`, `60`, `64`), each logged with its place. Refuses, before
/// anything, a first kind with fewer than three such kinds.
pub fn choose_arcade_opponents(shared: &RaceShared) -> Result<(), Refused> {
    use crate::race::race_shell::{index_of_kind, kind_unusable};
    if shared.vars.get().run_kind.get() != 2 {
        return Ok(());
    }
    let first: u32 = shared.vars.get().race_car_kinds[0].get();
    if opponent_kinds(shared, first)?.len() < 3 {
        return Err(BadInput("fewer than three opponents can be drawn").into());
    }
    let first_place = index_of_kind(shared, first)?;
    text(SEED_WORD, false);
    number(shared.tables().noise_seed.get(), true);
    text(OPPONENT_WORDS[0], false);
    number(first, false);
    text(OPPONENT_WORDS[1], false);
    number(first_place, true);
    shared
        .physics_tables
        .get()
        .noise_state
        .set(shared.tables().noise_seed.get());
    let mut taken = vec![first];
    for (n, at) in shared.vars.get().race_car_kinds[1..4].iter().enumerate() {
        let (kind, place) = loop {
            let random = if n == 0 {
                crate::scene::next_noise::next_noise(shared)
            } else {
                shared.random.next_random()
            };
            // Logic in `rc2k_core::race::cars::{drawn_kind, opponent_distance}`.
            let kind = rc2k_core::race::cars::drawn_kind(random);
            if kind_unusable(kind).0 {
                continue;
            }
            let place = index_of_kind(shared, kind)?;
            let (_, close) = rc2k_core::race::cars::opponent_distance(place, first_place);
            if !close || taken.contains(&kind) {
                continue;
            }
            break (kind, place);
        };
        taken.push(kind);
        at.set(kind);
        text(OPPONENT_WORDS[2 * n + 2], false);
        number(kind, false);
        text(OPPONENT_WORDS[2 * n + 3], false);
        number(place, true);
    }
    Ok(())
}

/// With `run_kind` 2, logs "SETUPARCADECARS", runs `choose_arcade_opponents` (the log indented
/// round it) and sets `race_cars` to 4; then sets each block's `car_kind` (`race_car_kinds` and the
/// seven after).
pub fn set_up_arcade_cars(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    if shared.vars.get().run_kind.get() == 2 {
        text(SETUP_ARCADE_WORD, true);
        let indent = crate::logfile::write_line::state::INDENT.atomic();
        indent.fetch_add(3, std::sync::atomic::Ordering::Relaxed);
        choose_arcade_opponents(shared)?;
        indent.fetch_sub(3, std::sync::atomic::Ordering::Relaxed);
        shared.view.get().race_cars.set(4u32);
    }
    for n in 0..8 {
        race.car_blocks().cars[n]
            .car_kind
            .set(shared.vars.get().race_car_kinds[n].get());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offset_of(field: Field) -> usize {
        let car = &CAR_BLOCKS.cars[0];
        let at = match field {
            Field::Word(field) => core::ptr::from_ref(field(car)).cast::<u8>(),
            Field::Float(field) => core::ptr::from_ref(field(car)).cast::<u8>(),
        };
        crate::physics::car_offsets_1999::offset_1999(car, at as usize)
    }

    /// Every selector sits where the core's old table of byte offsets had it.
    #[test]
    fn cleared_and_resets_sit_at_their_old_offsets() {
        let cleared: [usize; 22] = [
            0xFA8, 0xAC, 0x88, 0xA8, 0xA4, 0xA0, 0x29C, 0x98, 0x2AC, 0x2B0, 0x2A4, 0x2A8, 0x2B4,
            0x2A0, 0x428, 0x40C, 0x410, 0x414, 0x418, 0x41C, 0x420, 0x424,
        ];
        for (field, old) in CLEARED.iter().zip(cleared) {
            assert_eq!(offset_of(*field), old);
        }
        let resets: [(usize, u32); 6] = [
            (0x5FC, 0),
            (0xA0C, u32::MAX),
            (0xA14, u32::MAX),
            (0xA10, u32::MAX),
            (0xA1C, u32::MAX),
            (0xA18, u32::MAX),
        ];
        for ((field, v), (old, old_v)) in PLAYER_RESETS.iter().zip(resets) {
            assert_eq!((offset_of(*field), *v), (old, old_v));
        }
    }

    #[test]
    fn block_reset() {
        let w = reset_block(2, 3, 7);
        assert_eq!(w.len(), 25);
        assert_eq!((offset_of(w[0].0), w[0].1), (0x25C, 1));
        assert_eq!((offset_of(w[23].0), w[23].1), (0x38, 1));
        assert_eq!((offset_of(w[24].0), w[24].1), (0x8C, 700));
        assert_eq!(reset_block(3, 3, 0)[0].1, 0);
    }
}
