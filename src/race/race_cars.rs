//! Race init's "cars" (`race::set_up_cars`): by the kind of race one of
//! `race::set_up_mode_1_cars` / `race::set_up_mode_4_cars` / `race::set_up_race_cars` /
//! `race::set_up_mode_0_cars` (each: the car blocks reset, a two-player rule, the two players'
//! kinds) and the cars' data file read
//! (`race::read_stage_car_numbers`: per stage, three-digit numbers into `stage_numbers`). Here the
//! steps.

use crate::effects::cell_x87::CellX87;
use crate::errors::{BadInput, Refused};
use crate::math::x87_float as bh;
use crate::physics::car_store::Car;
#[cfg(test)]
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::session::Dword;
use crate::session::{PlainDword, PlainFloat};

/// The eight car blocks (fixed; `g_carBlocks` lists them), 0xFE0 apart.
/// Player n's (0, 1) kind, tyre and colours word: `car_kinds[n]`, `car_tyres[n]`,
/// `car_colours[n]`.
fn player_kinds(shared: &RaceShared, n: usize) -> [&Dword; 3] {
    let v = &shared.vars;
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

/// The block fields `reset_block` sets to 0, in the original's order.
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

/// What `race::reset_car_blocks` writes into car block `n` (field, value, in order):
/// `driver_kind` (+0x25C) 1 for the race's cars (`cars`, unsigned) else 0, the `CLEARED`
/// fields 0, `gearbox_kind` (+0x38) 1, `time_left` (+0x8C) the stage's number
/// (`stage_numbers`) x 100.
fn reset_block(n: u32, cars: u32, number: u32) -> Vec<(Field, u32)> {
    let mut writes: Vec<(Field, u32)> = vec![(Field::Word(|c| &c.driver_kind), (n < cars) as u32)];
    writes.extend(CLEARED.iter().map(|&field| (field, 0)));
    writes.push((Field::Word(|c| &c.gearbox_kind), 1));
    writes.push((Field::Word(|c| &c.time_left), number.wrapping_mul(100)));
    writes
}

/// `race::reset_car_blocks` (0x43658A): each of the eight car blocks (by `g_carBlocks`): +0x25C 1
/// for the race's cars (`g_raceCars`, unsigned) else 0, +0x38 1, +0x8C `stage_numbers` x 100, the
/// others in `CLEARED` 0; then each block's +0x24 (fixed addresses) its kind from `g_raceCarKinds`.
pub fn reset_car_blocks(race: &Race) -> Result<(), BadInput> {
    let physics = &race.physics;
    let shared = &race.shared;
    use rc2k_core::race::cars::CAR_BLOCKS as BLOCK_COUNT;
    {
        let cars: u32 = shared.view.race_cars.get();
        for n in 0..BLOCK_COUNT {
            let car = shared.race_car(n as usize)?;
            for (field, v) in reset_block(n, cars, physics.split_tables().stage_numbers[0].get()) {
                field.set(car, v);
            }
        }
        for n in 0..BLOCK_COUNT as usize {
            race.car_blocks().cars[n]
                .car_kind
                .set(shared.vars.race_car_kinds[n].get());
        }
    }
    Ok(())
}

/// `race::drop_absent_player` (0x436402): unless `run_kind` is 2 or 4, with `players` 2
/// (two players): the first player whose block's +0xB0 is not 0 (the first, else the second) is
/// out: +0x25C 0, +0xFA8 1.
pub fn drop_absent_player(race: &Race) {
    let shared = &race.shared;
    let blocks = race.car_blocks();
    // Converted: `rc2k_core::race::cars::absent_player`.
    {
        let out = rc2k_core::race::cars::absent_player(
            shared.vars.run_kind.get(),
            || shared.view.players.get(),
            |n| blocks.cars[n as usize].stop_state.get() != 0,
        );
        if let Some(n) = out {
            blocks.cars[n as usize].driver_kind.set(0u32);
            blocks.cars[n as usize].view_covered.set(1u32);
        }
    }
}

/// `race::set_player_car_kinds` (0x43670F): the two players' blocks' +0x24, +0x2C, +0x38 from
/// `PLAYER_KINDS`.
pub fn set_player_car_kinds(race: &Race) {
    let shared = &race.shared;
    for n in 0..2 {
        let car = &race.car_blocks().cars[n];
        let to = [&car.car_kind, &car.tyre_kind, &car.gearbox_kind];
        for (to, from) in to.into_iter().zip(player_kinds(shared, n)) {
            to.set(from.get());
        }
    }
}

/// The cars' data text from its start, as the original reads it (past the buffer's end
/// too).
fn text_data(shared: &RaceShared) -> crate::cells::StoreBytes<'_> {
    crate::cells::StoreBytes(&shared.tables().car_data_text)
}

/// `race::set_up_race_cars` (0x4364AD): `reset_car_blocks`, `drop_absent_player`,
/// `set_player_car_kinds`.
pub fn set_up_race_cars(race: &Race) -> Result<(), BadInput> {
    reset_car_blocks(race)?;
    {
        drop_absent_player(race);
        set_player_car_kinds(race);
    }
    Ok(())
}

// ---- the set-ups ----

/// The most numbers `g_stageCarNumbers` holds (`split_colour_count` follows it).
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

/// The log's calls (`log_write_l1`, `log_write_l1_nl`, `log_int_l1`, `log_int_l1_nl`,
/// `log_newline_l1`), each inside `pusha`.
fn text(text: &std::ffi::CStr, newline: bool) {
    crate::logfile::log_write::log_nul(text.to_bytes(), newline, 1);
}

fn number(value: u32, newline: bool) {
    crate::logfile::log_int::log_int(value as i32, newline, 1);
}

fn newline() {
    text(c"", true); // what `log_newline_l1` writes
}

fn word(n: usize) -> &'static std::ffi::CStr {
    SET_UP_WORDS[n]
}

/// `race::read_stage_car_numbers` (0x437971): `g_stageCarTotal` 0, `g_readBytes` 0x708; in
/// `g_carDataText` each 'l' or 'L' followed by two digits (`parse_2_digits`, 0 for none) is a
/// stage's line; on `g_stageNumber`'s, from three places on, three-digit numbers four apart
/// (`parse_3_digits`) into `g_stageCarNumbers` and summed into `g_stageCarTotal`, up to the text's
/// end or a non-digit. Refuses more than 0x100 numbers (worked out before anything is written).
pub fn read_stage_car_numbers(race: &Race) -> Result<(), Refused> {
    let physics = &race.physics;
    // Converted: `rc2k_core::race::stage::stage_car_numbers`.
    let len = TEXT_LEN as u32;
    let found = rc2k_core::race::stage::stage_car_numbers(
        &text_data(&race.shared),
        len,
        race.shared.vars.stage_number.get(),
    )?;
    crate::files::state::BYTES_READ.set(len);
    let mut total = 0u32;
    for (k, &n) in found.numbers.iter().enumerate() {
        physics.split_tables().stage_numbers[k].set(n);
        total = total.wrapping_add(n);
    }
    physics.split_tables().stage_car_total.set(total);
    Ok(())
}

/// `race::auto_set_up_car` (0x436F08): car block `b`'s set-up: +0x20, +0x3C, +0x30, +0x34 0x80,
/// +0x2C -1, +0x28 2; "autosetup", "t" and `g_stageNumber` logged; for stages 0x15 .. 0x4F the
/// set-up (stage - 0x15) / 10 x 6 + (stage - 0x15) % 10 of `auto_setups`: +0x30 its h, +0x34 its
/// s, +0x28 its +0xC less one, +0x2C its +8, each logged with its range, then a line break. Refuses
/// stage 0x14 (the original's `sub` then makes an index of 0x9999999B: a read far outside), before
/// anything.
pub fn auto_set_up_car(race: &Race, car: &Car) -> Result<(), Refused> {
    let course = &race.course;
    {
        // Converted: `rc2k_core::race::cars::set_up_pick`.
        let stage: u32 = race.shared.vars.stage_number.get();
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
                .map(crate::session::PlainDword::get)
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

/// `RaceState.settings.cars[0].steering_sensitivity` / `RaceState.settings.cars[1].steering_sensitivity` held for a block's +0x3C: below 0 is 0, 100 or more
/// 0x100, else times 2.56 (`fild` / `fmul` / `fistp`).
fn brake_value(race: &Race, n: usize) -> u32 {
    let course = &race.course;
    // Converted: `rc2k_core::race::cars::held_brake` (the product the x87's).
    rc2k_core::race::cars::held_brake(
        race.shared.state.settings.cars[n]
            .steering_sensitivity
            .get() as i32,
        |v| crate::math::x87::int_times_to_int(v, &course.numbers.get().brake_scale) as u32,
    )
}

/// `race::set_up_mode_0_cars` (0x4364BD): `reset_car_blocks`; with `session` 0,
/// `drop_absent_player`, `set_player_car_kinds`, `auto_set_up_car` of the two players, their +0x3C
/// from `RaceState.settings.cars[0].steering_sensitivity` / `RaceState.settings.cars[1].steering_sensitivity` (`brake_value`); else `auto_set_up_car` of every car of the
/// race (`pusha` / `popa` round each), then `set_player_car_kinds`. Refuses, with `session`
/// set, no cars or more than 8 (the original's `jnz` runs on), before anything.
pub fn set_up_mode_0_cars(race: &Race, net: &crate::net::owner::Net) -> Result<(), Refused> {
    {
        // Converted: `rc2k_core::race::cars::{every_car_set_up, held_brake}`.
        let cars: u32 = race.shared.view.race_cars.get();
        let all = rc2k_core::race::cars::every_car_set_up(net.vars.session.get() != 0, cars)?;
        reset_car_blocks(race)?;
        if !all {
            drop_absent_player(race);
            set_player_car_kinds(race);
            for n in 0..2 {
                auto_set_up_car(race, &race.car_blocks().cars[n as usize])?;
            }
            for n in 0..2 {
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

/// `race::set_up_mode_1_cars` (0x43645B): `reset_car_blocks`, `drop_absent_player`,
/// `set_player_car_kinds`; the two players' +0x88 from `RaceState.results.service_spent_current` / `RaceState.results.service_spent_first`.
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

/// `race::set_up_mode_4_cars` (0x43647F): `g_raceCars`, `players`, `views` 1;
/// `reset_car_blocks`, `drop_absent_player`, `set_player_car_kinds`.
pub fn set_up_mode_4_cars(race: &Race) -> Result<(), BadInput> {
    let shared = &race.shared;
    {
        shared.view.race_cars.set(1u32);
        shared.view.players.set(1u32);
        shared.view.views.set(1u32);
        reset_car_blocks(race)?;
        drop_absent_player(race);
        set_player_car_kinds(race);
    }
    Ok(())
}

// ---- the times, mode 2, the whole step ----

/// `race::clear_race_tables` (0x4632D4): entries 9 .. 0 of the thirteen tables 0 (four of them
/// overlap, one dword apart: the points' tables from entries 0 .. 3).
pub fn clear_race_tables(shared: &RaceShared) {
    let w = &shared.work;
    let tables: [(&[Dword], usize); 13] = [
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

/// "numcheckposits =", "difficultyindex=", "time percentage=", "total time =", "extra
/// time =", "chkp", "orig time", "car time".
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

/// `race::set_car_times` (0x4377D0): `kind_factor` the factor of the first car's kind (held to 0 ..
/// 0x16, `fld` / `fstp`); for stages 0x15 .. 0x4E the difficulty (the stage's place in its rally of
/// ten, plus 6 x the rally's level), its time percentage (`time_percentages`), the extra time
/// (percentage x `g_stageCarTotal` / 100) into `extra_time` and added to the first number; each
/// of the `split_colour_count` numbers times the factor; all logged. Stages below 0x15 go the first way
/// too, their index wrapping below the table (a read of the tables before it, as the original's).
/// Refuses, with a stage, a count of 0 or more than 0x100 (the original's `jnz` runs on / past the
/// numbers), before anything.
pub fn set_car_times(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let w = |n: usize| TIME_WORDS[n];
    {
        let stage: u32 = race.shared.vars.stage_number.get();
        let count: u32 = race.physics.split_tables().split_colour_count.get();
        if stage < rc2k_core::race::stage::UNTIMED_STAGES
            && (count == 0 || count as usize > MOST_NUMBERS)
        {
            return Err(BadInput("a checkpoint count of 0 or past the numbers").into());
        }
        // Converted: `rc2k_core::race::stage::{timed_kind, rally_of, difficulty}`; the
        // factor's float steps stay the x87's (whole numbers rounded from them).
        use rc2k_core::race::stage::{difficulty, rally_of, timed_kind};
        let kind = timed_kind(race.shared.vars.race_car_kinds[0].get() as i32);
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
        let percentage = crate::session::read_on_in(
            race.shared.tables(),
            &race.shared.tables().time_percentages,
            level as usize,
            "the time percentages",
        )?;
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
/// "ARCADE GAME MODE INIT", "rallynum", "stagenum", "NUM arcadecars", "bestpos", "arc
/// start pos (1-n)", "numcarsbehind", " - cartypes ".
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

/// `race::set_up_mode_2_cars` (0x43613A): the arcade race. Logged, from the rally's tables the
/// cars' count (`arcade_cars`), `start_share_last`, `start_share_first`, the start place (the count with
/// `arcade_kind` 2 or the rally's first stage), the best place; one player, four cars:
/// `reset_car_blocks`, `drop_absent_player`, cars 1 .. 3 +0x3C 0x80 and +0x25C 0,
/// `set_player_car_kinds`; from the noise seed `noise_seed`, each car a type 1 .. 3 (never twice
/// running) and a number (skipping the players' +0xAA8), the numbers then shuffled;
/// `clear_race_tables`; four `auto_set_up_car`s; the player's +0x3C (`brake_value`), +0x66C,
/// +0x65C, +0x660 0, +0x658 the cars behind; the types logged. Refuses a count of 0 or more than 89
/// (the `jnz` runs on / past the tables), before anything.
pub fn set_up_mode_2_cars(race: &Race) -> Result<(), Refused> {
    let w = |n: usize| ARCADE_WORDS[n];
    let rally: u32 = race.shared.vars.race_level.get();
    // The arcade tables hold six by rally (`arcade_cars_by_rally` ..) or stage (`arcade_best_place_by_stage`),
    // back to back: an index past them reads on into the next, as the original does.
    let pick = |table: &[crate::session::PlainDword; 6], n: u32| {
        crate::session::read_on_in(race.shared.tables(), table, n as usize, "the arcade tables")
    };
    let by_rally = |table: &[crate::session::PlainDword; 6]| pick(table, rally);
    let cars: u32 = by_rally(&race.shared.tables().arcade_cars_by_rally)?;
    if cars == 0 || cars > MOST_ARCADE_CARS {
        return Err(BadInput("an arcade car count of 0 or past the tables").into());
    }
    text(w(0), false);
    text(w(1), false);
    number(rally, false);
    text(w(2), false);
    let stage_in: u32 = race.shared.vars.race_stage.get();
    number(stage_in, false);
    race.shared.tables().arcade_cars.set(cars);
    race.shared
        .tables()
        .start_share_last
        .set(f32::from_bits(by_rally(
            &race.shared.tables().start_share_last_by_rally,
        )?));
    race.shared
        .tables()
        .start_share_first
        .set(f32::from_bits(by_rally(
            &race.shared.tables().start_share_first_by_rally,
        )?));
    // Converted: `rc2k_core::race::cars::{arcade_start_is_last, arcade_number,
    // arcade_type_draw, arcade_swap_draw}`.
    use crate::scene::next_noise::next_noise;
    use rc2k_core::race::cars::{
        arcade_number, arcade_start_is_last, arcade_swap_draw, arcade_type_draw,
    };
    if arcade_start_is_last(race.physics.split_tables().arcade_kind.get(), stage_in) {
        race.shared.tables().start_place.set(cars);
    }
    race.shared.tables().best_place.set(pick(
        &race.shared.tables().arcade_best_place_by_stage,
        stage_in,
    )?);
    text(w(3), false);
    number(cars, false);
    text(w(4), false);
    number(race.shared.tables().best_place.get(), false);
    text(w(5), false);
    number(race.shared.tables().start_place.get(), true);
    race.shared.view.views.set(1u32);
    race.shared.view.players.set(1u32);
    race.shared.view.race_cars.set(4u32);
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
        .noise_state
        .set(race.shared.tables().noise_seed.get());
    // Each car a type (never the last one's again) and a number.
    let mut last_type = u32::MAX;
    let mut number_next = 1u32;
    for car in 0..cars {
        let kind = loop {
            let first: u32 = race.car_blocks().cars[0].start_slot_value.get();
            let second = (race.shared.view.players.get() != 1)
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

/// `race::set_up_cars` (0x43603C): race init's "cars": "gamemode" and `run_kind` logged; both
/// players' +0x5FC 0 and +0xA0C .. +0xA1C -1; then by the mode: 2 the numbers, the times, the
/// arcade set-up, `kept_times` (0x40 dwords) 0; 1, 3, 0 (with the numbers first), 4 their
/// set-ups; others nothing.
pub fn set_up_cars(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    text(c"gamemode", false);
    number(race.shared.vars.run_kind.get(), true);
    // Converted: `rc2k_core::race::cars::cars_steps`.
    use rc2k_core::race::cars::{CarsStep, cars_steps};
    for n in 0..2 {
        for (field, v) in PLAYER_RESETS {
            field.set(race.shared.race_car(n as usize)?, v);
        }
    }
    for &step in cars_steps(race.shared.vars.run_kind.get()) {
        match step {
            CarsStep::Numbers => read_stage_car_numbers(race)?,
            CarsStep::Times => set_car_times(race, scene)?,
            CarsStep::SetUp(0) => set_up_mode_0_cars(race, net)?,
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

const SEED_WORD_RVA: &std::ffi::CStr = c"seed"; // "seed"
/// "p1", "power", "p2", "power", "p3", "power", "p4", "power".
const OPPONENT_WORDS: [&std::ffi::CStr; 8] = [
    c"p1", c"power", c"p2", c"power", c"p3", c"power", c"p4", c"power",
];
const SETUP_ARCADE_WORD_RVA: &std::ffi::CStr = c"SETUPARCADECARS"; // "SETUPARCADECARS"

/// The kinds a draw (0 .. 0x21) can give that `choose_arcade_opponents` takes: usable
/// (`kind_unusable` clear) and within 5 places (`index_of_kind`) of the first car's.
fn opponent_kinds(shared: &RaceShared, first: u32) -> Result<Vec<u32>, BadInput> {
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

/// `race::choose_arcade_opponents` (0x449B79): with `run_kind` 2, the first car's kind's place
/// (`index_of_kind`), logged ("seed", "p1", "power"); from the noise seed (`noise_seed`; then
/// `math::next_random`) kinds drawn (x 0x22 >> 32) until one is usable, within 5 places of the
/// first's and not taken: the second, third and fourth cars' (`RaceVars.race_car_kinds[1]`, `60`, `64`), each
/// logged with its place. Refuses, before anything, a first kind with fewer than three such kinds
/// (the original draws for ever).
pub fn choose_arcade_opponents(shared: &RaceShared) -> Result<(), Refused> {
    use crate::race::race_shell::{index_of_kind, kind_unusable};
    if shared.vars.run_kind.get() != 2 {
        return Ok(());
    }
    let first: u32 = shared.vars.race_car_kinds[0].get();
    if opponent_kinds(shared, first)?.len() < 3 {
        return Err(BadInput("fewer than three opponents can be drawn").into());
    }
    let first_place = index_of_kind(shared, first)?;
    text(SEED_WORD_RVA, false);
    number(shared.tables().noise_seed.get(), true);
    text(OPPONENT_WORDS[0], false);
    number(first, false);
    text(OPPONENT_WORDS[1], false);
    number(first_place, true);
    shared
        .physics_tables
        .noise_state
        .set(shared.tables().noise_seed.get());
    let mut taken = vec![first];
    for (n, at) in shared.vars.race_car_kinds[1..4].iter().enumerate() {
        let (kind, place) = loop {
            let random = if n == 0 {
                crate::scene::next_noise::next_noise(shared)
            } else {
                crate::math::next_random::next_random()
            };
            // Converted: `rc2k_core::race::cars::{drawn_kind, opponent_distance}`.
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

/// `race::set_up_arcade_cars` (0x4468FB): with `run_kind` 2, "SETUPARCADECARS" logged,
/// `choose_arcade_opponents` (the log indented round it), `g_raceCars` 4; then each block's +0x24
/// its kind (`g_raceCarKinds` and the seven after).
pub fn set_up_arcade_cars(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    if shared.vars.run_kind.get() == 2 {
        text(SETUP_ARCADE_WORD_RVA, true);
        let indent = crate::logfile::write_line::state::INDENT.atomic();
        indent.fetch_add(3, std::sync::atomic::Ordering::Relaxed);
        choose_arcade_opponents(shared)?;
        indent.fetch_sub(3, std::sync::atomic::Ordering::Relaxed);
        shared.view.race_cars.set(4u32);
    }
    for n in 0..8 {
        race.car_blocks().cars[n]
            .car_kind
            .set(shared.vars.race_car_kinds[n].get());
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
        at as usize - core::ptr::from_ref(car) as usize
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
