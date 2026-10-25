//! Race init's "cars" (`sub_43603C`, IDA 2026-10-17): by the kind of race one of
//! `sub_43645B` / `sub_43647F` / `sub_4364AD` / `sub_4364BD` (each: the car blocks reset,
//! a two-player rule, the two players' kinds) and the cars' data file read
//! (`sub_437971`: per stage, three-digit numbers into `dword_601430`). Here the steps.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::race::objects::{CAR_BLOCKS_RVA, CARS_RVA};
use crate::race::state_store::RACE_STATE;

/// The eight car blocks (fixed; `g_carBlocks` lists them), 0xFE0 apart.
pub const BLOCK_0: usize = 0x70F3EC;
pub const BLOCK_STEP: usize = 0xFE0;
pub const MODE_RVA: usize = rva(0x62389C); // dword_62389C
pub const PLAYERS_RVA: usize = rva(0x63DB94); // dword_63DB94
pub const SPEEDS_RVA: usize = rva(0x601430); // dword_601430: the stage's numbers from the file
/// g_raceCarKinds (0x623858) and the seven after it: the cars' kinds.
pub const CAR_KINDS_RVA: usize = rva(0x623858);
/// The two players' second and third kinds: (dword_6235FC, dword_623878) and
/// (dword_623600, dword_62387C).
pub const PLAYER_KINDS: [[usize; 3]; 2] = [
    [rva(0x623858), rva(0x6235FC), rva(0x623878)],
    [rva(0x62385C), rva(0x623600), rva(0x62387C)],
];
/// The text the digits are read from (`byte_6047B8`, 0x708 bytes).
pub const TEXT_RVA: usize = rva(0x6047B8);
pub const TEXT_LEN: usize = 0x708;

fn block(n: usize) -> usize {
    rva(BLOCK_0 + n * BLOCK_STEP)
}

/// `race::reset_car_blocks` (0x43658A): each of the eight car blocks (by `g_carBlocks`):
/// +0x25C 1 for the race's cars (`g_raceCars`, unsigned) else 0, +0x38 1, +0x8C
/// `dword_601430` x 100, the others in `CLEARED` 0; then each block's +0x24 (fixed
/// addresses) its kind from `g_raceCarKinds`.
pub unsafe fn reset_car_blocks() {
    // Converted: `rc2k_core::race::cars::reset_block`.
    use rc2k_core::race::cars::{CAR_BLOCKS, reset_block};
    unsafe {
        let cars: u32 = game_read(CARS_RVA);
        for n in 0..CAR_BLOCKS {
            let b: u32 = game_read(CAR_BLOCKS_RVA + 4 * n as usize);
            for (off, v) in reset_block(n, cars, game_read(SPEEDS_RVA)) {
                ((b + off) as *mut u32).write_unaligned(v);
            }
        }
        for n in 0..CAR_BLOCKS as usize {
            game_write(block(n) + 0x24, game_read::<u32>(CAR_KINDS_RVA + 4 * n));
        }
    }
}

/// `race::drop_absent_player` (0x436402): unless `dword_62389C` is 2 or 4, with
/// `dword_63DB94` 2 (two players): the first player whose block's +0xB0 is not 0 (the
/// first, else the second) is out: +0x25C 0, +0xFA8 1. Every register kept.
pub unsafe fn drop_absent_player() {
    // Converted: `rc2k_core::race::cars::absent_player`.
    unsafe {
        let out = rc2k_core::race::cars::absent_player(
            game_read(MODE_RVA),
            || game_read(PLAYERS_RVA),
            |n| game_read::<u32>(block(n as usize) + 0xB0) != 0,
        );
        if let Some(n) = out {
            game_write(block(n as usize) + 0x25C, 0u32);
            game_write(block(n as usize) + 0xFA8, 1u32);
        }
    }
}

/// `race::set_player_car_kinds` (0x43670F): the two players' blocks' +0x24, +0x2C, +0x38
/// from `PLAYER_KINDS`. Every register kept.
pub unsafe fn set_player_car_kinds() {
    for (n, kinds) in PLAYER_KINDS.iter().enumerate() {
        for (k, &from) in kinds.iter().enumerate() {
            unsafe { game_write(block(n) + [0x24, 0x2C, 0x38][k], game_read::<u32>(from)) };
        }
    }
}

/// The cars' data text from its start, as the original reads it (past the buffer's end
/// too).
fn text_data() -> crate::bridge::Raw {
    unsafe { crate::bridge::Raw::new(game_addr(TEXT_RVA) as *const u8) }
}

/// `race::set_up_race_cars` (0x4364AD): `reset_car_blocks`, `drop_absent_player`,
/// `set_player_car_kinds`.
pub unsafe fn set_up_race_cars() {
    unsafe {
        reset_car_blocks();
        drop_absent_player();
        set_player_car_kinds();
    }
}

// ---- the set-ups ----

pub const STAGE_RVA: usize = crate::race::stage_prefs::STAGE_RVA; // g_stageNumber
pub const STAGE_TOTAL_RVA: usize = rva(0x601C4C); // g_stageCarTotal
/// The most numbers `g_stageCarNumbers` holds (`dword_601830` follows it).
pub const MOST_NUMBERS: usize = rc2k_core::race::stage::MOST_NUMBERS;
/// dword_6043FC: the automatic set-ups, 0x10 bytes each (h, s, g5, g4 + 1).
pub const SET_UPS_RVA: usize = rva(0x6043FC);
const BRAKE_SCALE_RVA: usize = rva(0x5C5E00); // flt_5C5E00, 2.56
pub const PLAYERS_2_RVA: usize = rva(0x63DB98); // dword_63DB98
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
    unsafe { crate::logfile::log_write::log_write(at as *const u8, newline, 1) };
}

fn number(value: u32, newline: bool) {
    crate::logfile::log_int::log_int(value as i32, newline, 1);
}

fn newline() {
    text(c"".as_ptr() as u32, true); // what `log_newline_l1` writes
}

fn word(n: usize) -> u32 {
    SET_UP_WORDS[n].as_ptr() as usize as u32
}

/// `race::read_stage_car_numbers` (0x437971): `g_stageCarTotal` 0, `g_readBytes` 0x708;
/// in `g_carDataText` each 'l' or 'L' followed by two digits (`parse_2_digits`, 0 for
/// none) is a stage's line; on `g_stageNumber`'s, from three places on, three-digit
/// numbers four apart (`parse_3_digits`) into `g_stageCarNumbers` and summed into
/// `g_stageCarTotal`, up to the text's end or a non-digit. Refuses more than 0x100
/// numbers (worked out before anything is written).
pub unsafe fn read_stage_car_numbers() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::stage::stage_car_numbers`.
    let len = TEXT_LEN as u32;
    unsafe {
        let found =
            rc2k_core::race::stage::stage_car_numbers(&text_data(), len, game_read(STAGE_RVA))?;
        crate::files::state::BYTES_READ.store(len, std::sync::atomic::Ordering::Relaxed);
        let mut total = 0u32;
        for (k, &n) in found.numbers.iter().enumerate() {
            game_write(SPEEDS_RVA + 4 * k, n);
            total = total.wrapping_add(n);
        }
        game_write(STAGE_TOTAL_RVA, total);
    }
    Ok(())
}

/// `race::auto_set_up_car` (0x436F08): car block `b`'s set-up: +0x20, +0x3C, +0x30, +0x34
/// 0x80, +0x2C -1, +0x28 2; "autosetup", "t" and `g_stageNumber` logged; for stages 0x15
/// .. 0x4F the set-up (stage - 0x15) / 10 x 6 + (stage - 0x15) % 10 of `dword_6043FC`:
/// +0x30 its h, +0x34 its s, +0x28 its +0xC less one, +0x2C its +8, each logged with its
/// range, then a line break. Refuses stage 0x14 (the original's `sub` then makes an index
/// of 0x9999999B: a read far outside), before anything.
pub unsafe fn auto_set_up_car(b: u32) -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::race::cars::set_up_pick`.
        let stage: u32 = game_read(STAGE_RVA);
        let pick = rc2k_core::race::cars::set_up_pick(stage)?;
        let put = |off: u32, v: u32| ((b.wrapping_add(off)) as *mut u32).write_unaligned(v);
        for off in [0x20, 0x3C, 0x30, 0x34] {
            put(off, 0x80);
        }
        put(0x2C, u32::MAX);
        put(0x28, 2);
        text(word(0), false);
        text(word(1), false);
        number(stage, false);
        let Some(pick) = pick else {
            return Ok(());
        };
        text(word(2), false);
        number(pick.index, false);
        let set_up = (game_addr(SET_UPS_RVA) as u32).wrapping_add(pick.index << 4);
        let at = |k: u32| ((set_up + k) as *const u32).read_unaligned();
        for (value, off, words) in [
            (at(0), 0x30, [3, 4]),
            (at(4), 0x34, [5, 6]),
            (at(0xC).wrapping_sub(1), 0x28, [7, 8]),
            (at(8), 0x2C, [9, 10]),
        ] {
            put(off, value);
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
unsafe fn brake_value(n: usize) -> u32 {
    // Converted: `rc2k_core::race::cars::held_brake` (the product the x87's).
    unsafe {
        rc2k_core::race::cars::held_brake(
            RACE_STATE.settings.cars[n].steering_sensitivity.get() as i32,
            |v| crate::math::x87::int_times_to_int(v, game_read(BRAKE_SCALE_RVA)) as u32,
        )
    }
}

/// `race::set_up_mode_0_cars` (0x4364BD): `reset_car_blocks`; with `dword_52A910` 0,
/// `drop_absent_player`, `set_player_car_kinds`, `auto_set_up_car` of the two players,
/// their +0x3C from `dword_75AD2C` / `dword_75AE98` (`brake_value`); else
/// `auto_set_up_car` of every car of the race (`pusha` / `popa` round each), then
/// `set_player_car_kinds`. Refuses, with `dword_52A910` set, no cars or more than 8
/// (the original's `jnz` runs on), before anything.
pub unsafe fn set_up_mode_0_cars() -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::race::cars::{every_car_set_up, held_brake}`.
        let cars: u32 = game_read(CARS_RVA);
        let all = rc2k_core::race::cars::every_car_set_up(
            crate::net::store::NET.session.get() != 0,
            cars,
        )?;
        reset_car_blocks();
        if !all {
            drop_absent_player();
            set_player_car_kinds();
            for n in 0..2 {
                auto_set_up_car(game_addr(block(n)) as u32)?;
            }
            for n in 0..2 {
                game_write(block(n) + 0x3C, brake_value(n));
            }
        } else {
            for n in 0..cars {
                auto_set_up_car(game_read(CAR_BLOCKS_RVA + 4 * n as usize))?;
            }
            set_player_car_kinds();
        }
    }
    Ok(())
}

/// `race::set_up_mode_1_cars` (0x43645B): `reset_car_blocks`, `drop_absent_player`,
/// `set_player_car_kinds`; the two players' +0x88 from `dword_75B004` / `dword_75B008`.
pub unsafe fn set_up_mode_1_cars() {
    unsafe {
        reset_car_blocks();
        drop_absent_player();
        set_player_car_kinds();
        for (n, from) in [
            &RACE_STATE.results.word_288_current,
            &RACE_STATE.results.word_288_first,
        ]
        .into_iter()
        .enumerate()
        {
            game_write(block(n) + 0x88, from.get());
        }
    }
}

/// `race::set_up_mode_4_cars` (0x43647F): `g_raceCars`, `dword_63DB94`, `dword_63DB98`
/// 1; `reset_car_blocks`, `drop_absent_player`, `set_player_car_kinds`.
pub unsafe fn set_up_mode_4_cars() {
    unsafe {
        game_write(CARS_RVA, 1u32);
        game_write(PLAYERS_RVA, 1u32);
        game_write(PLAYERS_2_RVA, 1u32);
        reset_car_blocks();
        drop_absent_player();
        set_player_car_kinds();
    }
}

// ---- the times, mode 2, the whole step ----

/// The ten-entry tables `clear_race_tables` clears (`dword_71C010` ..).
const RACE_TABLES: [usize; 13] = [
    0x71C010, 0x71C038, 0x71BFA8, 0x71BFD0, 0x71C088, 0x71C0E8, 0x71C0EC, 0x71C0F0, 0x71C0F4,
    0x71C188, 0x71C18C, 0x71C190, 0x71C194,
];

/// `race::clear_race_tables` (0x4632D4): entries 9 .. 0 of the thirteen tables 0 (four
/// of them overlap, one dword apart). Every register kept (`pusha` / `popa`).
pub unsafe fn clear_race_tables() {
    for k in (0..10).rev() {
        for &table in &RACE_TABLES {
            unsafe { game_write(rva(table) + 4 * k, 0u32) };
        }
    }
}

const KIND_FACTORS_RVA: usize = rva(0x60475C); // flt_60475C: 0x17 floats by car kind
pub const KIND_FACTOR_RVA: usize = rva(0x601C64); // flt_601C64
const PERCENTAGES_RVA: usize = rva(0x6046CC); // dword_6046CC: by difficulty
pub const EXTRA_TIME_RVA: usize = rva(0x601C50); // dword_601C50
pub const CHECK_POSITS_RVA: usize = rva(0x601830); // dword_601830 ("numcheckposits")
const HUNDRED_RVA: usize = rva(0x5C5D14); // flt_5C5D14, 100.0
const SCRATCH_RVA: usize = crate::race::best_line::SCRATCH_RVA; // dword_62400C
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
pub unsafe fn set_car_times() -> Result<(), Refused> {
    let w = |n: usize| TIME_WORDS[n].as_ptr() as usize as u32;
    unsafe {
        let stage: u32 = game_read(STAGE_RVA);
        let count: u32 = game_read(CHECK_POSITS_RVA);
        if stage < rc2k_core::race::stage::UNTIMED_STAGES
            && (count == 0 || count as usize > MOST_NUMBERS)
        {
            return Err(BadInput("a checkpoint count of 0 or past the numbers").into());
        }
        // Converted: `rc2k_core::race::stage::{timed_kind, rally_of, difficulty}`; the
        // factor's float steps stay the x87's (whole numbers rounded from them).
        use rc2k_core::race::stage::{difficulty, rally_of, timed_kind};
        let kind = timed_kind(game_read(CAR_KINDS_RVA));
        let factor = crate::math::x87::loaded(game_read(KIND_FACTORS_RVA + 4 * kind as usize));
        game_write(KIND_FACTOR_RVA, factor);
        let Some(rally) = rally_of(stage) else {
            return Ok(());
        };
        let level = difficulty(stage, rally);
        text(w(0), false);
        number(count, false);
        text(w(1), false);
        number(level, false);
        let at = (game_addr(PERCENTAGES_RVA) as u32).wrapping_add(level.wrapping_mul(4));
        let percentage = (at as *const u32).read_unaligned();
        text(w(2), false);
        number(percentage, false);
        game_write(SCRATCH_RVA, percentage);
        let total: i32 = game_read(STAGE_TOTAL_RVA);
        let extra = crate::math::x87_load::int_times_int_over_to_int(
            percentage as i32,
            total,
            game_read(HUNDRED_RVA),
        );
        game_write(EXTRA_TIME_RVA, extra);
        text(w(3), false);
        number(total as u32, false);
        text(w(4), false);
        number(extra as u32, true);
        add(SPEEDS_RVA, extra as u32);
        let mut k = 0u32;
        loop {
            let at = SPEEDS_RVA + 4 * k as usize;
            text(w(5), false);
            number(k, false);
            text(w(6), false);
            number(game_read(at), false);
            let n = crate::math::x87::int_times_to_int(game_read(at), factor);
            game_write(at, n);
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

unsafe fn add(at: usize, by: u32) {
    unsafe { game_write(at, game_read::<u32>(at).wrapping_add(by)) };
}

const RALLY_RVA: usize = rva(0x6238CC); // dword_6238CC ("rallynum")
const STAGE_IN_RALLY_RVA: usize = rva(0x6238D0); // dword_6238D0 ("stagenum")
const ARCADE_CARS_RVA: usize = rva(0x6051B4); // dword_6051B4 ("NUM arcadecars")
const ARCADE_A_RVA: usize = rva(0x6051A4); // dword_6051A4
const ARCADE_B_RVA: usize = rva(0x6051A8); // dword_6051A8
const START_POS_RVA: usize = rva(0x6051BC); // dword_6051BC ("arc start pos (1-n)")
const BEST_POS_RVA: usize = rva(0x6051C0); // dword_6051C0 ("bestpos")
const ARCADE_KIND_RVA: usize = rva(0x60101C); // dword_60101C
const NOISE_SEED_RVA: usize = rva(0x60519C); // dword_60519C
const NOISE_STATE_RVA: usize = rva(0x614C54); // g_noiseState
const ARCADE_KINDS_RVA: usize = rva(0x604ECC); // dword_604ECC: each car's type, 1 .. 3
const ARCADE_ORDER_RVA: usize = rva(0x605034); // dword_605034: each car's number
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
pub unsafe fn set_up_mode_2_cars() -> Result<(), Refused> {
    let w = |n: usize| ARCADE_WORDS[n].as_ptr() as usize as u32;
    unsafe {
        let rally: u32 = game_read(RALLY_RVA);
        let cars: u32 = game_read(rva(0x60466C) + 4 * rally as usize);
        if cars == 0 || cars > MOST_ARCADE_CARS {
            return Err(BadInput("an arcade car count of 0 or past the tables").into());
        }
        text(w(0), false);
        text(w(1), false);
        number(rally, false);
        text(w(2), false);
        let stage_in: u32 = game_read(STAGE_IN_RALLY_RVA);
        number(stage_in, false);
        game_write(ARCADE_CARS_RVA, cars);
        game_write(
            ARCADE_A_RVA,
            game_read::<u32>(rva(0x60469C) + 4 * rally as usize),
        );
        game_write(
            ARCADE_B_RVA,
            game_read::<u32>(rva(0x6046B4) + 4 * rally as usize),
        );
        // Converted: `rc2k_core::race::cars::{arcade_start_is_last, arcade_number,
        // arcade_type_draw, arcade_swap_draw}`.
        use crate::scene::next_noise::next_noise;
        use rc2k_core::race::cars::{
            arcade_number, arcade_start_is_last, arcade_swap_draw, arcade_type_draw,
        };
        if arcade_start_is_last(game_read(ARCADE_KIND_RVA), stage_in) {
            game_write(START_POS_RVA, cars);
        }
        game_write(
            BEST_POS_RVA,
            game_read::<u32>(rva(0x604684) + 4 * stage_in as usize),
        );
        text(w(3), false);
        number(cars, false);
        text(w(4), false);
        number(game_read(BEST_POS_RVA), false);
        text(w(5), false);
        number(game_read(START_POS_RVA), true);
        game_write(PLAYERS_2_RVA, 1u32);
        game_write(PLAYERS_RVA, 1u32);
        game_write(CARS_RVA, 4u32);
        reset_car_blocks();
        drop_absent_player();
        for n in 1..4 {
            game_write(block(n) + 0x3C, 0x80u32);
        }
        for n in 1..4 {
            game_write(block(n) + 0x25C, 0u32);
        }
        set_player_car_kinds();
        game_write(NOISE_STATE_RVA, game_read::<u32>(NOISE_SEED_RVA));
        // Each car a type (never the last one's again) and a number.
        let mut last_type = u32::MAX;
        let mut number_next = 1u32;
        for car in 0..cars {
            let kind = loop {
                let first: u32 = game_read(block(0) + 0xAA8);
                let second = (game_read::<u32>(PLAYERS_RVA) != 1)
                    .then(|| game_read::<u32>(block(1) + 0xAA8));
                number_next = arcade_number(number_next, first, second);
                let (_, kind) = arcade_type_draw(next_noise(), car);
                if kind != last_type {
                    break kind;
                }
            };
            last_type = kind;
            game_write(ARCADE_KINDS_RVA + 4 * car as usize, kind + 1);
            game_write(ARCADE_ORDER_RVA + 4 * car as usize, number_next);
            number_next += 1;
        }
        // The numbers shuffled.
        for car in 0..cars {
            let mine: u32 = game_read(ARCADE_ORDER_RVA + 4 * car as usize);
            let (_, other_car) = arcade_swap_draw(next_noise(), cars);
            let other: u32 = game_read(ARCADE_ORDER_RVA + 4 * other_car as usize);
            game_write(ARCADE_ORDER_RVA + 4 * car as usize, other);
            game_write(ARCADE_ORDER_RVA + 4 * other_car as usize, mine);
        }
        clear_race_tables();
        for n in 0..4 {
            auto_set_up_car(game_addr(block(n)) as u32)?;
        }
        game_write(block(0) + 0x3C, brake_value(0));
        for off in [0x66C, 0x65C, 0x660] {
            game_write(block(0) + off, 0u32);
        }
        let behind = cars.wrapping_sub(game_read(START_POS_RVA));
        game_write(block(0) + 0x658, behind);
        text(w(6), false);
        number(behind, false);
        text(w(7), false);
        for car in 0..cars {
            number(game_read(ARCADE_KINDS_RVA + 4 * car as usize), false);
        }
        newline();
    }
    Ok(())
}

/// `race::set_up_cars` (0x43603C): race init's "cars": "gamemode" and `dword_62389C`
/// logged; both players' +0x5FC 0 and +0xA0C .. +0xA1C -1; then by the mode: 2 the
/// numbers, the times, the arcade set-up, `dword_601C68` (0x40 dwords) 0; 1, 3, 0 (with
/// the numbers first), 4 their set-ups; others nothing. The registers it leaves are read
/// by no caller (conversion phase 3, 2026-10-24).
pub unsafe fn set_up_cars() -> Result<(), Refused> {
    unsafe {
        text(c"gamemode".as_ptr() as usize as u32, false);
        number(game_read(MODE_RVA), true);
        // Converted: `rc2k_core::race::cars::{PLAYER_RESETS, cars_steps}`.
        use rc2k_core::race::cars::{CarsStep, PLAYER_RESETS, cars_steps};
        for n in 0..2 {
            for (off, v) in PLAYER_RESETS {
                game_write(block(n) + off as usize, v);
            }
        }
        for &step in cars_steps(game_read(MODE_RVA)) {
            match step {
                CarsStep::Numbers => read_stage_car_numbers()?,
                CarsStep::Times => set_car_times()?,
                CarsStep::SetUp(0) => set_up_mode_0_cars()?,
                CarsStep::SetUp(1) => set_up_mode_1_cars(),
                CarsStep::SetUp(2) => set_up_mode_2_cars()?,
                CarsStep::SetUp(3) => set_up_race_cars(),
                CarsStep::SetUp(_) => set_up_mode_4_cars(),
                CarsStep::ClearArcadeTable => {
                    for k in 0..0x40 {
                        game_write(rva(0x601C68) + 4 * k, 0u32);
                    }
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
unsafe fn opponent_kinds(first: u32) -> Vec<u32> {
    use crate::race::race_shell::{index_of_kind, kind_unusable};
    unsafe {
        let first_place = index_of_kind(first);
        (0..0x22u32)
            .filter(|&k| {
                !kind_unusable(k).0
                    && rc2k_core::race::cars::opponent_distance(index_of_kind(k), first_place).1
                    && k != first
            })
            .collect()
    }
}

/// `race::choose_arcade_opponents` (0x449B79): with `dword_62389C` 2, the first car's
/// kind's place (`index_of_kind`), logged ("seed", "p1", "power"); from the noise seed
/// (`dword_60519C`; then `math::next_random`) kinds drawn (x 0x22 >> 32) until one is
/// usable, within 5 places of the first's and not taken: the second, third and fourth
/// cars' (`dword_62385C`, `60`, `64`), each logged with its place. Refuses, before
/// anything, a first kind with fewer than three such kinds (the original draws for ever).
pub unsafe fn choose_arcade_opponents() -> Result<(), Refused> {
    use crate::race::race_shell::{index_of_kind, kind_unusable};
    unsafe {
        if game_read::<u32>(MODE_RVA) != 2 {
            return Ok(());
        }
        let first: u32 = game_read(CAR_KINDS_RVA);
        if opponent_kinds(first).len() < 3 {
            return Err(BadInput("fewer than three opponents can be drawn").into());
        }
        let first_place = index_of_kind(first);
        text(SEED_WORD_RVA.as_ptr() as usize as u32, false);
        number(game_read(NOISE_SEED_RVA), true);
        text(OPPONENT_WORDS[0].as_ptr() as usize as u32, false);
        number(first, false);
        text(OPPONENT_WORDS[1].as_ptr() as usize as u32, false);
        number(first_place, true);
        game_write(NOISE_STATE_RVA, game_read::<u32>(NOISE_SEED_RVA));
        let mut taken = vec![first];
        for (n, at) in [rva(0x62385C), rva(0x623860), rva(0x623864)]
            .into_iter()
            .enumerate()
        {
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
                let place = index_of_kind(kind);
                let (_, close) = rc2k_core::race::cars::opponent_distance(place, first_place);
                if !close || taken.contains(&kind) {
                    continue;
                }
                break (kind, place);
            };
            taken.push(kind);
            game_write(at, kind);
            text(OPPONENT_WORDS[2 * n + 2].as_ptr() as usize as u32, false);
            number(kind, false);
            text(OPPONENT_WORDS[2 * n + 3].as_ptr() as usize as u32, false);
            number(place, true);
        }
    }
    Ok(())
}

/// `race::set_up_arcade_cars` (0x4468FB): with `dword_62389C` 2, "SETUPARCADECARS"
/// logged, `choose_arcade_opponents` (the log indented round it), `g_raceCars` 4; then
/// each block's +0x24 its kind (`g_raceCarKinds` and the seven after). The original keeps
/// every register (liveness): the route leaves its caller's as they were.
pub unsafe fn set_up_arcade_cars() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(MODE_RVA) == 2 {
            text(SETUP_ARCADE_WORD_RVA.as_ptr() as usize as u32, true);
            let at = crate::logfile::write_line::indent_place() as *mut u32; // g_logIndent
            at.write_unaligned(at.read_unaligned().wrapping_add(3));
            choose_arcade_opponents()?;
            at.write_unaligned(at.read_unaligned().wrapping_sub(3));
            game_write(CARS_RVA, 4u32);
        }
        for n in 0..8 {
            game_write(block(n) + 0x24, game_read::<u32>(CAR_KINDS_RVA + 4 * n));
        }
    }
    Ok(())
}
