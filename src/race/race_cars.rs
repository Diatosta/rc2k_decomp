//! Race init's "cars" (`sub_43603C`, IDA 2026-10-17): by the kind of race one of
//! `sub_43645B` / `sub_43647F` / `sub_4364AD` / `sub_4364BD` (each: the car blocks reset,
//! a two-player rule, the two players' kinds) and the cars' data file read
//! (`sub_437971`: per stage, three-digit numbers into `dword_601430`). Here the steps.

use crate::errors::{BadInput, Refused};
use crate::physics::split_tables_store::SPLIT_TABLES;
use crate::physics::tables_store::PHYSICS;
use crate::pointers::GamePtr;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::numbers_store::NUMBERS;
use crate::race::state_store::RACE_STATE;
use crate::race::tables_store::RACE_TABLES;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;
use crate::session::Dword;

/// The eight car blocks (fixed; `g_carBlocks` lists them), 0xFE0 apart.
/// Player n's (0, 1) kind, tyre and colours word: `car_kinds[n]`, `car_tyres[n]`,
/// `car_colours[n]`.
fn player_kinds(n: usize) -> [&'static Dword; 3] {
    let v = &RACE_VARS;
    [&v.race_car_kinds[n], &v.car_tyres[n], &v.car_colours[n]]
}
/// The text the digits are read from (`byte_6047B8`, 0x708 bytes).
pub const TEXT_LEN: usize = 0x708;

/// `race::reset_car_blocks` (0x43658A): each of the eight car blocks (by `g_carBlocks`):
/// +0x25C 1 for the race's cars (`g_raceCars`, unsigned) else 0, +0x38 1, +0x8C
/// `dword_601430` x 100, the others in `CLEARED` 0; then each block's +0x24 (fixed
/// addresses) its kind from `g_raceCarKinds`.
pub fn reset_car_blocks() -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::cars::reset_block`.
    use rc2k_core::race::cars::{CAR_BLOCKS as BLOCK_COUNT, reset_block};
    {
        let cars: u32 = RACE_VIEW.race_cars.bits();
        for n in 0..BLOCK_COUNT {
            let b: u32 = crate::race::work_store::RACE_WORK.car_block(n as usize);
            let car = GamePtr::to(crate::race::trap::car_at(b)?);
            for (off, v) in reset_block(n, cars, SPLIT_TABLES.stage_numbers[0].get()) {
                car.offset(off).write(v, "a car block's word")?;
            }
        }
        for n in 0..BLOCK_COUNT as usize {
            CAR_BLOCKS.cars[n]
                .car_kind
                .set_bits(RACE_VARS.race_car_kinds[n].get());
        }
    }
    Ok(())
}

/// `race::drop_absent_player` (0x436402): unless `dword_62389C` is 2 or 4, with
/// `dword_63DB94` 2 (two players): the first player whose block's +0xB0 is not 0 (the
/// first, else the second) is out: +0x25C 0, +0xFA8 1. Every register kept.
pub fn drop_absent_player() {
    // Converted: `rc2k_core::race::cars::absent_player`.
    {
        let out = rc2k_core::race::cars::absent_player(
            RACE_VARS.run_kind.bits(),
            || RACE_VIEW.players.bits(),
            |n| CAR_BLOCKS.cars[n as usize].stop_state.bits() != 0,
        );
        if let Some(n) = out {
            CAR_BLOCKS.cars[n as usize].driver_kind.set_bits(0u32);
            CAR_BLOCKS.cars[n as usize].view_covered.set_bits(1u32);
        }
    }
}

/// `race::set_player_car_kinds` (0x43670F): the two players' blocks' +0x24, +0x2C, +0x38
/// from `PLAYER_KINDS`. Every register kept.
pub fn set_player_car_kinds() {
    for n in 0..2 {
        let car = &CAR_BLOCKS.cars[n];
        let to = [&car.car_kind, &car.tyre_kind, &car.gearbox_kind];
        for (to, from) in to.into_iter().zip(player_kinds(n)) {
            to.set(from.get());
        }
    }
}

/// The cars' data text from its start, as the original reads it (past the buffer's end
/// too).
fn text_data() -> Result<crate::pointers::GameBytes, BadInput> {
    GamePtr::to(&RACE_TABLES.car_data_text).bytes("the cars' data text")
}

/// `race::set_up_race_cars` (0x4364AD): `reset_car_blocks`, `drop_absent_player`,
/// `set_player_car_kinds`.
pub fn set_up_race_cars() -> Result<(), BadInput> {
    reset_car_blocks()?;
    {
        drop_absent_player();
        set_player_car_kinds();
    }
    Ok(())
}

// ---- the set-ups ----

/// The most numbers `g_stageCarNumbers` holds (`dword_601830` follows it).
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
fn text(at: u32, newline: bool) {
    crate::logfile::log_write::log_at(at, newline, 1);
}

fn number(value: u32, newline: bool) {
    crate::logfile::log_int::log_int(value as i32, newline, 1);
}

fn newline() {
    text(crate::pointers::game_address(c"".as_ptr()), true); // what `log_newline_l1` writes
}

fn word(n: usize) -> u32 {
    crate::pointers::game_address(SET_UP_WORDS[n].as_ptr())
}

/// `race::read_stage_car_numbers` (0x437971): `g_stageCarTotal` 0, `g_readBytes` 0x708;
/// in `g_carDataText` each 'l' or 'L' followed by two digits (`parse_2_digits`, 0 for
/// none) is a stage's line; on `g_stageNumber`'s, from three places on, three-digit
/// numbers four apart (`parse_3_digits`) into `g_stageCarNumbers` and summed into
/// `g_stageCarTotal`, up to the text's end or a non-digit. Refuses more than 0x100
/// numbers (worked out before anything is written).
pub fn read_stage_car_numbers() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::stage::stage_car_numbers`.
    let len = TEXT_LEN as u32;
    let found = rc2k_core::race::stage::stage_car_numbers(
        &text_data()?,
        len,
        RACE_VARS.stage_number.bits(),
    )?;
    crate::files::state::BYTES_READ.store(len, std::sync::atomic::Ordering::Relaxed);
    let mut total = 0u32;
    for (k, &n) in found.numbers.iter().enumerate() {
        SPLIT_TABLES.stage_numbers[k].set(n);
        total = total.wrapping_add(n);
    }
    SPLIT_TABLES.stage_car_total.set(total);
    Ok(())
}

/// `race::auto_set_up_car` (0x436F08): car block `b`'s set-up: +0x20, +0x3C, +0x30, +0x34
/// 0x80, +0x2C -1, +0x28 2; "autosetup", "t" and `g_stageNumber` logged; for stages 0x15
/// .. 0x4F the set-up (stage - 0x15) / 10 x 6 + (stage - 0x15) % 10 of `dword_6043FC`:
/// +0x30 its h, +0x34 its s, +0x28 its +0xC less one, +0x2C its +8, each logged with its
/// range, then a line break. Refuses stage 0x14 (the original's `sub` then makes an index
/// of 0x9999999B: a read far outside), before anything.
pub fn auto_set_up_car(b: u32) -> Result<(), Refused> {
    {
        // Converted: `rc2k_core::race::cars::set_up_pick`.
        let stage: u32 = RACE_VARS.stage_number.bits();
        let pick = rc2k_core::race::cars::set_up_pick(stage)?;
        let car = GamePtr::to(crate::race::trap::car_at(b)?);
        let put = |off: u32, v: u32| car.offset(off).write(v, "a car block's set-up");
        for off in [0x20, 0x3C, 0x30, 0x34] {
            put(off, 0x80)?;
        }
        put(0x2C, u32::MAX)?;
        put(0x28, 2)?;
        text(word(0), false);
        text(word(1), false);
        number(stage, false);
        let Some(pick) = pick else {
            return Ok(());
        };
        text(word(2), false);
        number(pick.index, false);
        let set_up =
            GamePtr::to(&crate::race::league_store::LEAGUE.auto_setups).offset(pick.index << 4);
        let at = |k: u32| set_up.offset(k).read::<u32>("the auto set-ups");
        for (value, off, words) in [
            (at(0)?, 0x30, [3, 4]),
            (at(4)?, 0x34, [5, 6]),
            (at(0xC)?.wrapping_sub(1), 0x28, [7, 8]),
            (at(8)?, 0x2C, [9, 10]),
        ] {
            put(off, value)?;
            text(word(words[0]), false);
            number(value, false);
            text(word(words[1]), false);
        }
        newline();
    }
    Ok(())
}

/// `dword_75AD2C` / `dword_75AE98` held for a block's +0x3C: below 0 is 0, 100 or more
/// 0x100, else times 2.56 (`fild` / `fmul` / `fistp`).
fn brake_value(n: usize) -> u32 {
    // Converted: `rc2k_core::race::cars::held_brake` (the product the x87's).
    rc2k_core::race::cars::held_brake(
        RACE_STATE.settings.cars[n].steering_sensitivity.get() as i32,
        |v| crate::math::x87::int_times_to_int(v, NUMBERS.brake_scale.bits()) as u32,
    )
}

/// `race::set_up_mode_0_cars` (0x4364BD): `reset_car_blocks`; with `dword_52A910` 0,
/// `drop_absent_player`, `set_player_car_kinds`, `auto_set_up_car` of the two players,
/// their +0x3C from `dword_75AD2C` / `dword_75AE98` (`brake_value`); else
/// `auto_set_up_car` of every car of the race (`pusha` / `popa` round each), then
/// `set_player_car_kinds`. Refuses, with `dword_52A910` set, no cars or more than 8
/// (the original's `jnz` runs on), before anything.
pub fn set_up_mode_0_cars() -> Result<(), Refused> {
    {
        // Converted: `rc2k_core::race::cars::{every_car_set_up, held_brake}`.
        let cars: u32 = RACE_VIEW.race_cars.bits();
        let all = rc2k_core::race::cars::every_car_set_up(
            crate::net::store::NET.session.get() != 0,
            cars,
        )?;
        reset_car_blocks()?;
        if !all {
            drop_absent_player();
            set_player_car_kinds();
            for n in 0..2 {
                auto_set_up_car(crate::race::blocks_store::block_addr(n as usize))?;
            }
            for n in 0..2 {
                CAR_BLOCKS.cars[n]
                    .steering_sensitivity
                    .set_bits(brake_value(n));
            }
        } else {
            for n in 0..cars {
                auto_set_up_car(crate::race::work_store::RACE_WORK.car_block(n as usize))?;
            }
            set_player_car_kinds();
        }
    }
    Ok(())
}

/// `race::set_up_mode_1_cars` (0x43645B): `reset_car_blocks`, `drop_absent_player`,
/// `set_player_car_kinds`; the two players' +0x88 from `dword_75B004` / `dword_75B008`.
pub fn set_up_mode_1_cars() -> Result<(), BadInput> {
    {
        reset_car_blocks()?;
        drop_absent_player();
        set_player_car_kinds();
        for (n, from) in [
            &RACE_STATE.results.service_spent_current,
            &RACE_STATE.results.service_spent_first,
        ]
        .into_iter()
        .enumerate()
        {
            CAR_BLOCKS.cars[n].stage_time.set_bits(from.get());
        }
    }
    Ok(())
}

/// `race::set_up_mode_4_cars` (0x43647F): `g_raceCars`, `dword_63DB94`, `dword_63DB98`
/// 1; `reset_car_blocks`, `drop_absent_player`, `set_player_car_kinds`.
pub fn set_up_mode_4_cars() -> Result<(), BadInput> {
    {
        RACE_VIEW.race_cars.set_bits(1u32);
        RACE_VIEW.players.set_bits(1u32);
        RACE_VIEW.views.set_bits(1u32);
        reset_car_blocks()?;
        drop_absent_player();
        set_player_car_kinds();
    }
    Ok(())
}

// ---- the times, mode 2, the whole step ----

/// `race::clear_race_tables` (0x4632D4): entries 9 .. 0 of the thirteen tables 0 (four
/// of them overlap, one dword apart: the points' tables from entries 0 .. 3). Every
/// register kept (`pusha` / `popa`).
pub fn clear_race_tables() {
    let w = &RACE_WORK;
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

/// `race::set_car_times` (0x4377D0): `flt_601C64` the factor of the first car's kind
/// (held to 0 .. 0x16, `fld` / `fstp`); for stages 0x15 .. 0x4E the difficulty (the
/// stage's place in its rally of ten, plus 6 x the rally's level), its time percentage
/// (`dword_6046CC`), the extra time (percentage x `g_stageCarTotal` / 100) into
/// `dword_601C50` and added to the first number; each of the `dword_601830` numbers times
/// the factor; all logged. Stages below 0x15 go the first way too, their index wrapping
/// below the table (a read of the tables before it, as the original's). Refuses, with a
/// stage, a count of 0 or more than 0x100 (the original's `jnz` runs on / past the
/// numbers), before anything.
pub fn set_car_times() -> Result<(), Refused> {
    let w = |n: usize| crate::pointers::game_address(TIME_WORDS[n].as_ptr());
    {
        let stage: u32 = RACE_VARS.stage_number.bits();
        let count: u32 = SPLIT_TABLES.split_colour_count.get();
        if stage < rc2k_core::race::stage::UNTIMED_STAGES
            && (count == 0 || count as usize > MOST_NUMBERS)
        {
            return Err(BadInput("a checkpoint count of 0 or past the numbers").into());
        }
        // Converted: `rc2k_core::race::stage::{timed_kind, rally_of, difficulty}`; the
        // factor's float steps stay the x87's (whole numbers rounded from them).
        use rc2k_core::race::stage::{difficulty, rally_of, timed_kind};
        let kind = timed_kind(RACE_VARS.race_car_kinds[0].bits() as i32);
        let factor = crate::math::x87::loaded(RACE_TABLES.kind_time_factors[kind as usize].bits());
        SPLIT_TABLES.kind_factor.set_bits(factor);
        let Some(rally) = rally_of(stage) else {
            return Ok(());
        };
        let level = difficulty(stage, rally);
        text(w(0), false);
        number(count, false);
        text(w(1), false);
        number(level, false);
        let percentage = crate::session::read_on_in(
            &RACE_TABLES,
            &RACE_TABLES.time_percentages,
            level as usize,
            "the time percentages",
        )?;
        text(w(2), false);
        number(percentage, false);
        DRAW.scratch.set_bits(percentage);
        let total: i32 = SPLIT_TABLES.stage_car_total.get() as i32;
        let extra = crate::math::x87_load::int_times_int_over_to_int(
            percentage as i32,
            total,
            NUMBERS.hundred.bits(),
        );
        SPLIT_TABLES.extra_time.set(extra as u32);
        text(w(3), false);
        number(total as u32, false);
        text(w(4), false);
        number(extra as u32, true);
        SPLIT_TABLES.stage_numbers[0].set(
            SPLIT_TABLES.stage_numbers[0]
                .get()
                .wrapping_add(extra as u32),
        );
        let mut k = 0u32;
        loop {
            let at = &SPLIT_TABLES.stage_numbers[k as usize];
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

/// The room `dword_604ECC` has (`dword_605030` follows it).
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

/// `race::set_up_mode_2_cars` (0x43613A): the arcade race. Logged, from the rally's tables
/// the cars' count (`dword_6051B4`), `dword_6051A4`, `dword_6051A8`, the start place (the
/// count with `dword_60101C` 2 or the rally's first stage), the best place; one player,
/// four cars: `reset_car_blocks`, `drop_absent_player`, cars 1 .. 3 +0x3C 0x80 and +0x25C
/// 0, `set_player_car_kinds`; from the noise seed `dword_60519C`, each car a type 1 .. 3
/// (never twice running) and a number (skipping the players' +0xAA8), the numbers then
/// shuffled; `clear_race_tables`; four `auto_set_up_car`s; the player's +0x3C
/// (`brake_value`), +0x66C, +0x65C, +0x660 0, +0x658 the cars behind; the types logged.
/// Refuses a count of 0 or more than 89 (the `jnz` runs on / past the tables), before
/// anything.
pub fn set_up_mode_2_cars() -> Result<(), Refused> {
    let w = |n: usize| crate::pointers::game_address(ARCADE_WORDS[n].as_ptr());
    let rally: u32 = RACE_VARS.race_level.bits();
    // The arcade tables hold six by rally (`dword_60466C` ..) or stage (`dword_604684`),
    // back to back: an index past them reads on into the next, as the original does.
    let pick = |table: &[crate::session::Dword; 6], n: u32| {
        crate::session::read_on_in(&RACE_TABLES, table, n as usize, "the arcade tables")
    };
    let by_rally = |table: &[crate::session::Dword; 6]| pick(table, rally);
    let cars: u32 = by_rally(&RACE_TABLES.arcade_cars_by_rally)?;
    if cars == 0 || cars > MOST_ARCADE_CARS {
        return Err(BadInput("an arcade car count of 0 or past the tables").into());
    }
    text(w(0), false);
    text(w(1), false);
    number(rally, false);
    text(w(2), false);
    let stage_in: u32 = RACE_VARS.race_stage.bits();
    number(stage_in, false);
    RACE_TABLES.arcade_cars.set_bits(cars);
    RACE_TABLES
        .arcade_a
        .set_bits(by_rally(&RACE_TABLES.arcade_a_by_rally)?);
    RACE_TABLES
        .arcade_b
        .set_bits(by_rally(&RACE_TABLES.arcade_b_by_rally)?);
    // Converted: `rc2k_core::race::cars::{arcade_start_is_last, arcade_number,
    // arcade_type_draw, arcade_swap_draw}`.
    use crate::scene::next_noise::next_noise;
    use rc2k_core::race::cars::{
        arcade_number, arcade_start_is_last, arcade_swap_draw, arcade_type_draw,
    };
    if arcade_start_is_last(SPLIT_TABLES.arcade_kind.get(), stage_in) {
        RACE_TABLES.start_place.set_bits(cars);
    }
    RACE_TABLES
        .best_place
        .set_bits(pick(&RACE_TABLES.arcade_best_place_by_stage, stage_in)?);
    text(w(3), false);
    number(cars, false);
    text(w(4), false);
    number(RACE_TABLES.best_place.bits(), false);
    text(w(5), false);
    number(RACE_TABLES.start_place.bits(), true);
    RACE_VIEW.views.set_bits(1u32);
    RACE_VIEW.players.set_bits(1u32);
    RACE_VIEW.race_cars.set_bits(4u32);
    reset_car_blocks()?;
    drop_absent_player();
    for n in 1..4 {
        CAR_BLOCKS.cars[n as usize]
            .steering_sensitivity
            .set_bits(0x80u32);
    }
    for n in 1..4 {
        CAR_BLOCKS.cars[n as usize].driver_kind.set_bits(0u32);
    }
    set_player_car_kinds();
    PHYSICS.noise_state.set(RACE_TABLES.noise_seed.bits());
    // Each car a type (never the last one's again) and a number.
    let mut last_type = u32::MAX;
    let mut number_next = 1u32;
    for car in 0..cars {
        let kind = loop {
            let first: u32 = CAR_BLOCKS.cars[0].start_slot_value.bits();
            let second =
                (RACE_VIEW.players.bits() != 1).then(|| CAR_BLOCKS.cars[1].start_slot_value.bits());
            number_next = arcade_number(number_next, first, second);
            let (_, kind) = arcade_type_draw(next_noise(), car);
            if kind != last_type {
                break kind;
            }
        };
        last_type = kind;
        RACE_TABLES.arcade_car_kinds[car as usize].set(kind + 1);
        RACE_TABLES.arcade_car_numbers[car as usize].set(number_next);
        number_next += 1;
    }
    // The numbers shuffled.
    for car in 0..cars {
        let mine: u32 = RACE_TABLES.arcade_car_numbers[car as usize].get();
        let (_, other_car) = arcade_swap_draw(next_noise(), cars);
        let other: u32 = RACE_TABLES.arcade_car_numbers[other_car as usize].get();
        RACE_TABLES.arcade_car_numbers[car as usize].set(other);
        RACE_TABLES.arcade_car_numbers[other_car as usize].set(mine);
    }
    clear_race_tables();
    for n in 0..4 {
        auto_set_up_car(crate::race::blocks_store::block_addr(n as usize))?;
    }
    CAR_BLOCKS.cars[0]
        .steering_sensitivity
        .set_bits(brake_value(0));
    let lead = &CAR_BLOCKS.cars[0];
    for cell in [&lead.pace_placed, &lead.pace_cars[0], &lead.pace_cars[1]] {
        cell.set(0);
    }
    let behind = cars.wrapping_sub(RACE_TABLES.start_place.bits());
    CAR_BLOCKS.cars[0].mode2_behind.set_bits(behind);
    text(w(6), false);
    number(behind, false);
    text(w(7), false);
    for car in 0..cars {
        number(RACE_TABLES.arcade_car_kinds[car as usize].get(), false);
    }
    newline();
    Ok(())
}

/// `race::set_up_cars` (0x43603C): race init's "cars": "gamemode" and `dword_62389C`
/// logged; both players' +0x5FC 0 and +0xA0C .. +0xA1C -1; then by the mode: 2 the
/// numbers, the times, the arcade set-up, `dword_601C68` (0x40 dwords) 0; 1, 3, 0 (with
/// the numbers first), 4 their set-ups; others nothing. The registers it leaves are read
/// by no caller (conversion phase 3, 2026-10-24).
pub fn set_up_cars() -> Result<(), Refused> {
    text(crate::pointers::game_address(c"gamemode".as_ptr()), false);
    number(RACE_VARS.run_kind.bits(), true);
    // Converted: `rc2k_core::race::cars::{PLAYER_RESETS, cars_steps}`.
    use rc2k_core::race::cars::{CarsStep, PLAYER_RESETS, cars_steps};
    for n in 0..2 {
        for (off, v) in PLAYER_RESETS {
            let b = crate::race::work_store::RACE_WORK.car_block(n as usize);
            GamePtr::to(crate::race::trap::car_at(b)?)
                .offset(off)
                .write(v, "a car block's word")?;
        }
    }
    for &step in cars_steps(RACE_VARS.run_kind.bits()) {
        match step {
            CarsStep::Numbers => read_stage_car_numbers()?,
            CarsStep::Times => set_car_times()?,
            CarsStep::SetUp(0) => set_up_mode_0_cars()?,
            CarsStep::SetUp(1) => set_up_mode_1_cars()?,
            CarsStep::SetUp(2) => set_up_mode_2_cars()?,
            CarsStep::SetUp(3) => set_up_race_cars()?,
            CarsStep::SetUp(_) => set_up_mode_4_cars()?,
            CarsStep::ClearArcadeTable => {
                for k in 0..0x40 {
                    SPLIT_TABLES.kept_times[k].set(0);
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
fn opponent_kinds(first: u32) -> Result<Vec<u32>, BadInput> {
    use crate::race::race_shell::{index_of_kind, kind_unusable};
    {
        let first_place = index_of_kind(first)?;
        let mut kinds = Vec::new();
        for k in 0..0x22u32 {
            if !kind_unusable(k).0
                && rc2k_core::race::cars::opponent_distance(index_of_kind(k)?, first_place).1
                && k != first
            {
                kinds.push(k);
            }
        }
        Ok(kinds)
    }
}

/// `race::choose_arcade_opponents` (0x449B79): with `dword_62389C` 2, the first car's
/// kind's place (`index_of_kind`), logged ("seed", "p1", "power"); from the noise seed
/// (`dword_60519C`; then `math::next_random`) kinds drawn (x 0x22 >> 32) until one is
/// usable, within 5 places of the first's and not taken: the second, third and fourth
/// cars' (`dword_62385C`, `60`, `64`), each logged with its place. Refuses, before
/// anything, a first kind with fewer than three such kinds (the original draws for ever).
pub fn choose_arcade_opponents() -> Result<(), Refused> {
    use crate::race::race_shell::{index_of_kind, kind_unusable};
    if RACE_VARS.run_kind.bits() != 2 {
        return Ok(());
    }
    let first: u32 = RACE_VARS.race_car_kinds[0].bits();
    if opponent_kinds(first)?.len() < 3 {
        return Err(BadInput("fewer than three opponents can be drawn").into());
    }
    let first_place = index_of_kind(first)?;
    text(crate::pointers::game_address(SEED_WORD_RVA.as_ptr()), false);
    number(RACE_TABLES.noise_seed.bits(), true);
    text(
        crate::pointers::game_address(OPPONENT_WORDS[0].as_ptr()),
        false,
    );
    number(first, false);
    text(
        crate::pointers::game_address(OPPONENT_WORDS[1].as_ptr()),
        false,
    );
    number(first_place, true);
    PHYSICS.noise_state.set(RACE_TABLES.noise_seed.bits());
    let mut taken = vec![first];
    for (n, at) in RACE_VARS.race_car_kinds[1..4].iter().enumerate() {
        let (kind, place) = loop {
            let random = if n == 0 {
                crate::scene::next_noise::next_noise()
            } else {
                crate::math::next_random::next_random()
            };
            // Converted: `rc2k_core::race::cars::{drawn_kind, opponent_distance}`.
            let kind = rc2k_core::race::cars::drawn_kind(random);
            if kind_unusable(kind).0 {
                continue;
            }
            let place = index_of_kind(kind)?;
            let (_, close) = rc2k_core::race::cars::opponent_distance(place, first_place);
            if !close || taken.contains(&kind) {
                continue;
            }
            break (kind, place);
        };
        taken.push(kind);
        at.set(kind);
        text(
            crate::pointers::game_address(OPPONENT_WORDS[2 * n + 2].as_ptr()),
            false,
        );
        number(kind, false);
        text(
            crate::pointers::game_address(OPPONENT_WORDS[2 * n + 3].as_ptr()),
            false,
        );
        number(place, true);
    }
    Ok(())
}

/// `race::set_up_arcade_cars` (0x4468FB): with `dword_62389C` 2, "SETUPARCADECARS"
/// logged, `choose_arcade_opponents` (the log indented round it), `g_raceCars` 4; then
/// each block's +0x24 its kind (`g_raceCarKinds` and the seven after). The original keeps
/// every register (liveness): the route leaves its caller's as they were.
pub fn set_up_arcade_cars() -> Result<(), Refused> {
    if RACE_VARS.run_kind.bits() == 2 {
        text(
            crate::pointers::game_address(SETUP_ARCADE_WORD_RVA.as_ptr()),
            true,
        );
        let indent = &crate::logfile::write_line::state::INDENT;
        indent.fetch_add(3, std::sync::atomic::Ordering::Relaxed);
        choose_arcade_opponents()?;
        indent.fetch_sub(3, std::sync::atomic::Ordering::Relaxed);
        RACE_VIEW.race_cars.set_bits(4u32);
    }
    for n in 0..8 {
        CAR_BLOCKS.cars[n]
            .car_kind
            .set_bits(RACE_VARS.race_car_kinds[n].get());
    }
    Ok(())
}
