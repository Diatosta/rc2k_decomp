//! Race init's "cars" (`sub_43603C`, IDA 2026-10-17): by the kind of race one of
//! `sub_43645B` / `sub_43647F` / `sub_4364AD` / `sub_4364BD` (each: the car blocks reset,
//! a two-player rule, the two players' kinds) and the cars' data file read
//! (`sub_437971`: per stage, three-digit numbers into `dword_601430`). Here the steps.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::race::objects::{CAR_BLOCKS_RVA, CARS_RVA};
use crate::service::harness::{Region, at};

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

/// The block offsets `reset_car_blocks` sets to 0 (+0x25C first; +0x38 gets 1).
const CLEARED: [usize; 26] = [
    0xFA8, 0xAC, 0x88, 0xA8, 0xA4, 0xA0, 0x29C, 0x98, 0x2AC, 0x2B0, 0x2A4, 0x2A8, 0x2B4, 0x2A0,
    0x428, 0x40C, 0x410, 0x414, 0x418, 0x41C, 0x420, 0x424, 0x25C, 0x38, 0x8C, 0x24,
];

/// `race::reset_car_blocks` (0x43658A): each of the eight car blocks (by `g_carBlocks`):
/// +0x25C 1 for the race's cars (`g_raceCars`, unsigned) else 0, +0x38 1, +0x8C
/// `dword_601430` x 100, the others in `CLEARED` 0; then each block's +0x24 (fixed
/// addresses) its kind from `g_raceCarKinds`. Leaves ecx 8, esi the last block, eax the
/// number x 100.
pub unsafe fn reset_car_blocks(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let cars: u32 = game_read(CARS_RVA);
        for n in 0..8u32 {
            let b: u32 = game_read(CAR_BLOCKS_RVA + 4 * n as usize);
            let put = |off: u32, v: u32| ((b + off) as *mut u32).write_unaligned(v);
            put(0x25C, (n < cars) as u32);
            for &off in &CLEARED[..22] {
                put(off as u32, 0);
            }
            put(0x38, 1);
            g.eax = game_read::<u32>(SPEEDS_RVA).wrapping_mul(100);
            put(0x8C, g.eax);
            g.esi = b;
        }
        g.ecx = 8;
        for n in 0..8 {
            game_write(block(n) + 0x24, game_read::<u32>(CAR_KINDS_RVA + 4 * n));
        }
    }
    g
}

/// What `reset_car_blocks` writes.
pub fn block_regions() -> Vec<Region> {
    let mut r = Vec::new();
    for n in 0..8u32 {
        let b: u32 = unsafe { game_read(CAR_BLOCKS_RVA + 4 * n as usize) };
        for &off in &CLEARED[..25] {
            r.push((b + off as u32, 4));
        }
        r.push(at(block(n as usize) + 0x24));
    }
    r
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

/// What `drop_absent_player` writes.
pub fn player_regions() -> Vec<Region> {
    (0..2)
        .flat_map(|n| [at(block(n) + 0x25C), at(block(n) + 0xFA8)])
        .collect()
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

/// What `set_player_car_kinds` writes.
pub fn kind_regions() -> Vec<Region> {
    (0..2)
        .flat_map(|n| [0x24, 0x2C, 0x38].map(|off| at(block(n) + off)))
        .collect()
}

/// The cars' data text from its start, as the original reads it (past the buffer's end
/// too).
fn text() -> crate::bridge::Raw {
    unsafe { crate::bridge::Raw::new(game_addr(TEXT_RVA) as *const u8) }
}

/// `race::parse_2_digits` (0x437A25): the two digits at text + ecx as a number; Err with
/// any not a digit (the original then gives eax 0, carry set).
pub unsafe fn parse_2_digits(at: u32) -> Option<u32> {
    // Converted: `rc2k_core::race::stage::parse_digits`.
    rc2k_core::race::stage::parse_digits(&text(), at as usize, 2)
}

/// `race::parse_3_digits` (0x4379EB): the three digits at text + ecx (the second's test
/// before the third's), as `parse_2_digits`.
pub unsafe fn parse_3_digits(at: u32) -> Option<u32> {
    // Converted: `rc2k_core::race::stage::parse_digits`.
    rc2k_core::race::stage::parse_digits(&text(), at as usize, 3)
}

/// `race::set_up_race_cars` (0x4364AD): `reset_car_blocks`, `drop_absent_player`,
/// `set_player_car_kinds`. Leaves the registers as the first.
pub unsafe fn set_up_race_cars(entry: &Regs) -> Result<Regs, Refused> {
    unsafe {
        let g = reset_car_blocks(entry);
        drop_absent_player();
        set_player_car_kinds();
        Ok(g)
    }
}

/// The game's eight blocks' first 0xFE0 bytes and the inputs, for saving.
pub fn saved_regions() -> Vec<Region> {
    let mut r: Vec<Region> = (0..8)
        .map(|n| (game_addr(block(n)) as u32, BLOCK_STEP))
        .collect();
    r.extend([
        at(MODE_RVA),
        at(PLAYERS_RVA),
        at(SPEEDS_RVA),
        at(CARS_RVA),
        (game_addr(CAR_KINDS_RVA) as u32, 32),
        at(PLAYER_KINDS[0][1]),
        at(PLAYER_KINDS[0][2]),
        at(PLAYER_KINDS[1][1]),
        at(PLAYER_KINDS[1][2]),
    ]);
    r
}

/// Fills the blocks with a pattern and sets the inputs of a case.
pub fn prepare(seed: u32, mode: u32, players: u32, cars: u32, absent: [u32; 2]) {
    unsafe {
        for n in 0..8 {
            let base = game_addr(block(n)) as *mut u32;
            for k in 0..BLOCK_STEP / 4 {
                base.add(k)
                    .write_unaligned((k as u32 * 0x0001_0003) ^ seed ^ n as u32);
            }
        }
        game_write(MODE_RVA, mode);
        game_write(PLAYERS_RVA, players);
        game_write(CARS_RVA, cars);
        game_write(SPEEDS_RVA, seed & 0xFFF);
        for n in 0..8 {
            game_write(CAR_KINDS_RVA + 4 * n, (0x100 + n as u32) ^ seed);
        }
        for (n, kinds) in PLAYER_KINDS.iter().enumerate() {
            game_write(kinds[1], 0x200 + n as u32);
            game_write(kinds[2], 0x300 + n as u32);
        }
        for (n, &a) in absent.iter().enumerate() {
            game_write(block(n) + 0xB0, a);
        }
    }
}

// ---- the set-ups ----

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The set-ups' callees by place (for the copies' stand-ins and `cars_part`): 1 .. 8
/// `set_up_cars`' steps and `clear_race_tables`, 9 .. 12 the four log entries, 13
/// `log_newline_l1`, 14 .. 17 the three steps above and `auto_set_up_car`. Place 18,
/// `scene::next_noise`, is not stood in for (the copies call the game's).
pub const CAR_CALLEES: &[(usize, u32)] = &[
    (0x437971, 1),
    (0x4377D0, 2),
    (0x43613A, 3),
    (0x43645B, 4),
    (0x4364AD, 5),
    (0x4364BD, 6),
    (0x43647F, 7),
    (0x4632D4, 8),
    (0x401427, 9),
    (0x40142B, 10),
    (0x401472, 11),
    (0x401476, 12),
    (0x4013DE, 13),
    (0x43658A, 14),
    (0x436402, 15),
    (0x43670F, 16),
    (0x436F08, 17),
    (0x449B79, 19),
];

pub const STAGE_RVA: usize = crate::race::stage_prefs::STAGE_RVA; // g_stageNumber
pub const STAGE_TOTAL_RVA: usize = rva(0x601C4C); // g_stageCarTotal
pub const READ_BYTES_RVA: usize = crate::race::course_data::READ_BYTES_RVA; // g_readBytes
/// The most numbers `g_stageCarNumbers` holds (`dword_601830` follows it).
pub const MOST_NUMBERS: usize = rc2k_core::race::stage::MOST_NUMBERS;
/// dword_6043FC: the automatic set-ups, 0x10 bytes each (h, s, g5, g4 + 1).
pub const SET_UPS_RVA: usize = rva(0x6043FC);
pub const ALL_AUTO_RVA: usize = rva(0x52A910); // dword_52A910
const BRAKES_RVA: [usize; 2] = [rva(0x75AD2C), rva(0x75AE98)]; // dword_75AD2C, dword_75AE98
const BRAKE_SCALE_RVA: usize = rva(0x5C5E00); // flt_5C5E00, 2.56
const MODE_1_VALUES_RVA: [usize; 2] = [rva(0x75B004), rva(0x75B008)]; // dword_75B004 / 8
pub const PLAYERS_2_RVA: usize = rva(0x63DB98); // dword_63DB98
/// "autosetup", "t", "off", "h", "(0-255)", "s", "(0-255)", "g", "(0-4)", "g", "(0-5)".
const SET_UP_WORDS: [usize; 11] = [
    rva(0x4DBA62),
    rva(0x4DBA6C),
    rva(0x4DBA6E),
    rva(0x4DBA72),
    rva(0x4DBA74),
    rva(0x4DBA7C),
    rva(0x4DBA7E),
    rva(0x4DBA86),
    rva(0x4DBA88),
    rva(0x4DBA8E),
    rva(0x4DBA90),
];
const EMPTY_TEXT_RVA: usize = rva(0x4E00B7); // what `log_newline_l1` writes

/// A log call inside `pusha`: eax the text or the number, the registers kept.
fn log(place: u32, eax: u32, g: &Regs, part: Part) -> Result<(), Refused> {
    let mut r = Regs { eax, ..*g };
    part(place, &mut r)
}

fn word(n: usize) -> u32 {
    game_addr(SET_UP_WORDS[n]) as u32
}

/// `race::read_stage_car_numbers` (0x437971): `g_stageCarTotal` 0, `g_readBytes` 0x708;
/// in `g_carDataText` each 'l' or 'L' followed by two digits (`parse_2_digits`, 0 for
/// none) is a stage's line; on `g_stageNumber`'s, from three places on, three-digit
/// numbers four apart (`parse_3_digits`) into `g_stageCarNumbers` and summed into
/// `g_stageCarTotal`, up to the text's end or a non-digit. Leaves ecx the place reached,
/// ebp the count, eax the last number parsed (0 for a failed one). Refuses more than
/// 0x100 numbers (worked out before anything is written).
pub unsafe fn read_stage_car_numbers(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::stage::stage_car_numbers`.
    let mut g = *entry;
    let len = TEXT_LEN as u32;
    unsafe {
        let found = rc2k_core::race::stage::stage_car_numbers(&text(), len, game_read(STAGE_RVA))?;
        g.ebp = found.numbers.len() as u32;
        g.ecx = found.place;
        if let Some(last) = found.last {
            g.eax = last;
        }
        game_write(READ_BYTES_RVA, len);
        let mut total = 0u32;
        for (k, &n) in found.numbers.iter().enumerate() {
            game_write(SPEEDS_RVA + 4 * k, n);
            total = total.wrapping_add(n);
        }
        game_write(STAGE_TOTAL_RVA, total);
    }
    Ok(g)
}

pub fn number_regions() -> Vec<Region> {
    vec![
        at(STAGE_TOTAL_RVA),
        at(READ_BYTES_RVA),
        (game_addr(SPEEDS_RVA) as u32, 4 * MOST_NUMBERS),
    ]
}

/// `race::auto_set_up_car` (0x436F08): car block esi's set-up: +0x20, +0x3C, +0x30, +0x34
/// 0x80, +0x2C -1, +0x28 2; "autosetup", "t" and `g_stageNumber` logged; for stages 0x15
/// .. 0x4F the set-up (stage - 0x15) / 10 x 6 + (stage - 0x15) % 10 of `dword_6043FC`:
/// +0x30 its h, +0x34 its s, +0x28 its +0xC less one, +0x2C its +8, each logged with its
/// range, then a line break. Leaves eax the last, ebx 10, edx the remainder, ebp the
/// set-up's address (without one: eax the stage). Refuses stage 0x14 (the original's
/// `sub` then makes an index of 0x9999999B: a read far outside), before anything.
pub unsafe fn auto_set_up_car(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let stage: u32 = game_read(STAGE_RVA);
        if stage == 0x14 {
            return Err(BadInput("stage 0x14: the set-up index runs outside").into());
        }
        let b = g.esi;
        let put = |off: u32, v: u32| ((b.wrapping_add(off)) as *mut u32).write_unaligned(v);
        for off in [0x20, 0x3C, 0x30, 0x34] {
            put(off, 0x80);
        }
        put(0x2C, u32::MAX);
        put(0x28, 2);
        log(9, word(0), &g, part)?;
        g.eax = stage;
        log(9, word(1), &g, part)?;
        log(11, g.eax, &g, part)?;
        if !(0x14..0x50).contains(&stage) {
            return Ok(g);
        }
        let v = stage - 0x15;
        g.ebx = 10;
        g.edx = v % 10;
        g.eax = (v / 10) * 6 + v % 10;
        log(9, word(2), &g, part)?;
        log(11, g.eax, &g, part)?;
        g.eax <<= 4;
        g.ebp = (game_addr(SET_UPS_RVA) as u32).wrapping_add(g.eax);
        let at = |k: u32| ((g.ebp + k) as *const u32).read_unaligned();
        for (value, off, words) in [
            (at(0), 0x30, [3, 4]),
            (at(4), 0x34, [5, 6]),
            (at(0xC).wrapping_sub(1), 0x28, [7, 8]),
            (at(8), 0x2C, [9, 10]),
        ] {
            g.eax = value;
            put(off, value);
            log(9, word(words[0]), &g, part)?;
            log(11, g.eax, &g, part)?;
            log(9, word(words[1]), &g, part)?;
        }
        log(13, g.eax, &g, part)?;
    }
    Ok(g)
}

/// `dword_75AD2C` / `dword_75AE98` held for a block's +0x3C: below 0 is 0, 100 or more
/// 0x100, else times 2.56 (`fild` / `fmul` / `fistp`).
unsafe fn brake_value(n: usize) -> u32 {
    unsafe {
        let v: i32 = game_read(BRAKES_RVA[n]);
        if v < 0 {
            0
        } else if v.wrapping_sub(100) >= 0 {
            0x100
        } else {
            crate::math::x87::int_times_to_int(v, game_read(BRAKE_SCALE_RVA)) as u32
        }
    }
}

/// `race::set_up_mode_0_cars` (0x4364BD): `reset_car_blocks`; with `dword_52A910` 0,
/// `drop_absent_player`, `set_player_car_kinds`, `auto_set_up_car` of the two players,
/// their +0x3C from `dword_75AD2C` / `dword_75AE98` (`brake_value`); else
/// `auto_set_up_car` of every car of the race (`pusha` / `popa` round each), then
/// `set_player_car_kinds`. Refuses, with `dword_52A910` set, no cars or more than 8
/// (the original's `jnz` runs on), before anything.
pub unsafe fn set_up_mode_0_cars(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let all = game_read::<u32>(ALL_AUTO_RVA) != 0;
        let cars: u32 = game_read(CARS_RVA);
        if all && (cars == 0 || cars > 8) {
            return Err(BadInput("no cars or more than eight").into());
        }
        part(14, &mut g)?;
        if !all {
            part(15, &mut g)?;
            part(16, &mut g)?;
            for n in 0..2 {
                g.esi = game_addr(block(n)) as u32;
                part(17, &mut g)?;
            }
            for n in 0..2 {
                game_write(block(n) + 0x3C, brake_value(n));
            }
        } else {
            for n in 0..cars {
                let mut c = Regs { ecx: n, ..g };
                c.esi = game_read(CAR_BLOCKS_RVA + 4 * n as usize);
                part(17, &mut c)?;
            }
            g.ecx = cars;
            part(16, &mut g)?;
        }
    }
    Ok(g)
}

/// `race::set_up_mode_1_cars` (0x43645B): `reset_car_blocks`, `drop_absent_player`,
/// `set_player_car_kinds`; the two players' +0x88 from `dword_75B004` / `dword_75B008`.
/// Leaves eax the second, the rest as the steps.
pub unsafe fn set_up_mode_1_cars(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        part(14, &mut g)?;
        part(15, &mut g)?;
        part(16, &mut g)?;
        for (n, &from) in MODE_1_VALUES_RVA.iter().enumerate() {
            g.eax = game_read(from);
            game_write(block(n) + 0x88, g.eax);
        }
    }
    Ok(g)
}

/// `race::set_up_mode_4_cars` (0x43647F): `g_raceCars`, `dword_63DB94`, `dword_63DB98`
/// 1; `reset_car_blocks`, `drop_absent_player`, `set_player_car_kinds`.
pub unsafe fn set_up_mode_4_cars(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        game_write(CARS_RVA, 1u32);
        game_write(PLAYERS_RVA, 1u32);
        game_write(PLAYERS_2_RVA, 1u32);
        part(14, &mut g)?;
        part(15, &mut g)?;
        part(16, &mut g)?;
    }
    Ok(g)
}

/// The set-ups' callees in the game: the Rust versions.
pub fn cars_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    unsafe {
        match place {
            9 | 10 => log_write(r.eax as *const u8, place == 10, 1),
            11 | 12 => log_int(r.eax as i32, place == 12, 1),
            13 => log_write(game_addr(EMPTY_TEXT_RVA) as *const u8, true, 1),
            14 => *r = reset_car_blocks(r),
            15 => drop_absent_player(),
            16 => set_player_car_kinds(),
            17 => *r = auto_set_up_car(r, &mut cars_part)?,
            1 => *r = read_stage_car_numbers(r)?,
            2 => *r = set_car_times(r, &mut cars_part)?,
            3 => *r = set_up_mode_2_cars(r, &mut cars_part)?,
            4 => *r = set_up_mode_1_cars(r, &mut cars_part)?,
            5 => *r = set_up_race_cars(r)?,
            6 => *r = set_up_mode_0_cars(r, &mut cars_part)?,
            7 => *r = set_up_mode_4_cars(r, &mut cars_part)?,
            8 => clear_race_tables(),
            19 => *r = choose_arcade_opponents(r, &mut cars_part)?,
            _ => r.eax = crate::scene::next_noise::next_noise(),
        }
    }
    Ok(())
}

/// What the set-ups write (beyond the blocks, which `saved_regions` covers) and read.
pub fn set_up_regions() -> Vec<Region> {
    let mut r = saved_regions();
    r.extend([
        at(STAGE_RVA),
        at(ALL_AUTO_RVA),
        at(BRAKES_RVA[0]),
        at(BRAKES_RVA[1]),
        at(MODE_1_VALUES_RVA[0]),
        at(MODE_1_VALUES_RVA[1]),
        at(PLAYERS_2_RVA),
    ]);
    r
}

/// The inputs of a set-up case beyond `prepare`'s.
pub fn prepare_set_up(stage: u32, all: u32, brakes: [i32; 2]) {
    unsafe {
        game_write(STAGE_RVA, stage);
        game_write(ALL_AUTO_RVA, all);
        game_write(BRAKES_RVA[0], brakes[0]);
        game_write(BRAKES_RVA[1], brakes[1]);
        game_write(MODE_1_VALUES_RVA[0], 0x1234u32);
        game_write(MODE_1_VALUES_RVA[1], 0x5678u32);
        game_write(PLAYERS_2_RVA, 7u32);
    }
}

/// One case on a copy whose calls note themselves (`race::mud::copy` with `CAR_CALLEES`):
/// the copy and the Rust version from `before`, after `prepare`; the registers left, the
/// calls noted and `regions` compared.
pub fn run_cars(
    t: &mut crate::framework::SelfTest,
    what: &str,
    copy: &crate::redirect::RedirectedCopy,
    regions: &[Region],
    before: Regs,
    prepare: &dyn Fn(),
    rust: &dyn Fn(&Regs) -> Result<Regs, Refused>,
) {
    use crate::race::level_init::take_called;
    use crate::service::harness::capture;
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    prepare();
    take_called();
    let mut ro = before;
    unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
    let theirs = (Some(regs(&ro)), take_called(), capture(regions));
    prepare();
    let mine = rust(&before);
    let mine = (
        mine.as_ref().ok().map(regs),
        take_called(),
        capture(regions),
    );
    t.check(theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
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

/// What `clear_race_tables` writes.
pub fn race_table_regions() -> Vec<Region> {
    RACE_TABLES
        .iter()
        .map(|&t| (game_addr(rva(t)) as u32, 40))
        .collect()
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
const TIME_WORDS: [usize; 8] = [
    rva(0x4DBA96),
    rva(0x4DBAA7),
    rva(0x4DBAB8),
    rva(0x4DBAC9),
    rva(0x4DBAD6),
    rva(0x4DBAE3),
    rva(0x4DBAE8),
    rva(0x4DBAF2),
];

/// `race::set_car_times` (0x4377D0): `flt_601C64` the factor of the first car's kind
/// (held to 0 .. 0x16, `fld` / `fstp`); for stages 0x15 .. 0x4E the difficulty (the
/// stage's place in its rally of ten, plus 6 x the rally's level), its time percentage
/// (`dword_6046CC`), the extra time (percentage x `g_stageCarTotal` / 100) into
/// `dword_601C50` and added to the first number; each of the `dword_601830` numbers times
/// the factor; all logged. Leaves eax the extra time, ebx the count (without a stage: eax
/// the stage). Stages below 0x15 go the first way too, their index wrapping below the
/// table (a read of the tables before it, as the original's). Refuses, with a stage, a
/// count of 0 or more than 0x100 (the original's `jnz` runs on / past the numbers),
/// before anything.
pub unsafe fn set_car_times(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    let w = |n: usize| game_addr(TIME_WORDS[n]) as u32;
    unsafe {
        let stage: u32 = game_read(STAGE_RVA);
        let count: u32 = game_read(CHECK_POSITS_RVA);
        if stage < 0x4F && (count == 0 || count as usize > MOST_NUMBERS) {
            return Err(BadInput("a checkpoint count of 0 or past the numbers").into());
        }
        let mut kind: i32 = game_read(CAR_KINDS_RVA);
        if kind < 0 {
            kind = 0;
        }
        if kind.wrapping_sub(0x16) >= 0 {
            kind = 0x16;
        }
        let factor = crate::math::x87::loaded(game_read(KIND_FACTORS_RVA + 4 * kind as usize));
        game_write(KIND_FACTOR_RVA, factor);
        g.eax = stage;
        let (base, level) = if stage < 0x1D {
            (0x15, 0)
        } else if stage < 0x27 {
            (0x1F, 3)
        } else if stage < 0x31 {
            (0x29, 5)
        } else if stage < 0x3B {
            (0x33, 1)
        } else if stage < 0x45 {
            (0x3D, 4)
        } else if stage < 0x4F {
            (0x47, 2)
        } else {
            return Ok(g);
        };
        g.ebx = level;
        g.eax = stage.wrapping_sub(base).wrapping_add(6 * level);
        log(9, w(0), &g, part)?;
        log(11, count, &g, part)?;
        log(9, w(1), &g, part)?;
        log(11, g.eax, &g, part)?;
        let at = (game_addr(PERCENTAGES_RVA) as u32).wrapping_add(g.eax.wrapping_mul(4));
        g.eax = (at as *const u32).read_unaligned();
        log(9, w(2), &g, part)?;
        log(11, g.eax, &g, part)?;
        game_write(SCRATCH_RVA, g.eax);
        let total: i32 = game_read(STAGE_TOTAL_RVA);
        let extra = crate::math::x87_load::int_times_int_over_to_int(
            g.eax as i32,
            total,
            game_read(HUNDRED_RVA),
        );
        game_write(EXTRA_TIME_RVA, extra);
        log(9, w(3), &g, part)?;
        log(11, total as u32, &g, part)?;
        log(9, w(4), &g, part)?;
        log(12, extra as u32, &g, part)?;
        g.eax = extra as u32;
        add(SPEEDS_RVA, g.eax);
        g.ebx = 0;
        loop {
            let at = SPEEDS_RVA + 4 * g.ebx as usize;
            log(9, w(5), &g, part)?;
            log(11, g.ebx, &g, part)?;
            log(9, w(6), &g, part)?;
            log(11, game_read(at), &g, part)?;
            let n = crate::math::x87::int_times_to_int(game_read(at), factor);
            game_write(at, n);
            log(9, w(7), &g, part)?;
            log(12, n as u32, &g, part)?;
            g.ebx += 1;
            if g.ebx == count {
                break;
            }
        }
    }
    Ok(g)
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
const ARCADE_WORDS: [usize; 8] = [
    rva(0x4DB9F4),
    rva(0x4DBA0A),
    rva(0x4DBA13),
    rva(0x4DBA1C),
    rva(0x4DBA2B),
    rva(0x4DBA33),
    rva(0x4DBA47),
    rva(0x4DBA55),
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
pub unsafe fn set_up_mode_2_cars(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    let w = |n: usize| game_addr(ARCADE_WORDS[n]) as u32;
    unsafe {
        let rally: u32 = game_read(RALLY_RVA);
        let cars: u32 = game_read(rva(0x60466C) + 4 * rally as usize);
        if cars == 0 || cars > MOST_ARCADE_CARS {
            return Err(BadInput("an arcade car count of 0 or past the tables").into());
        }
        log(9, w(0), &g, part)?;
        log(9, w(1), &g, part)?;
        log(11, rally, &g, part)?;
        log(9, w(2), &g, part)?;
        let stage_in: u32 = game_read(STAGE_IN_RALLY_RVA);
        log(11, stage_in, &g, part)?;
        g.ebx = rally;
        game_write(ARCADE_CARS_RVA, cars);
        g.eax = game_read(rva(0x60469C) + 4 * rally as usize);
        game_write(ARCADE_A_RVA, g.eax);
        g.eax = game_read(rva(0x6046B4) + 4 * rally as usize);
        game_write(ARCADE_B_RVA, g.eax);
        if game_read::<u32>(ARCADE_KIND_RVA) == 2 || stage_in == 0 {
            g.eax = cars;
            game_write(START_POS_RVA, cars);
        }
        g.ebx = stage_in;
        g.eax = game_read(rva(0x604684) + 4 * stage_in as usize);
        game_write(BEST_POS_RVA, g.eax);
        log(9, w(3), &g, part)?;
        log(11, cars, &g, part)?;
        log(9, w(4), &g, part)?;
        log(11, game_read(BEST_POS_RVA), &g, part)?;
        log(9, w(5), &g, part)?;
        log(12, game_read(START_POS_RVA), &g, part)?;
        game_write(PLAYERS_2_RVA, 1u32);
        game_write(PLAYERS_RVA, 1u32);
        game_write(CARS_RVA, 4u32);
        part(14, &mut g)?;
        part(15, &mut g)?;
        for n in 1..4 {
            game_write(block(n) + 0x3C, 0x80u32);
        }
        for n in 1..4 {
            game_write(block(n) + 0x25C, 0u32);
        }
        part(16, &mut g)?;
        game_write(NOISE_STATE_RVA, game_read::<u32>(NOISE_SEED_RVA));
        g.ebp = u32::MAX;
        g.ecx = 0;
        g.esi = 1;
        loop {
            loop {
                if g.esi == game_read::<u32>(block(0) + 0xAA8) {
                    g.esi += 1;
                }
                if game_read::<u32>(PLAYERS_RVA) != 1 && g.esi == game_read::<u32>(block(1) + 0xAA8)
                {
                    g.esi += 1;
                }
                part(18, &mut g)?;
                g.eax = g.eax.rotate_right(g.ecx & 0x1F);
                g.ebx = 3;
                let product = g.eax as u64 * 3;
                g.eax = product as u32;
                g.edx = (product >> 32) as u32;
                if g.edx != g.ebp {
                    break;
                }
            }
            g.ebp = g.edx;
            g.edx += 1;
            game_write(ARCADE_KINDS_RVA + 4 * g.ecx as usize, g.edx);
            game_write(ARCADE_ORDER_RVA + 4 * g.ecx as usize, g.esi);
            g.esi += 1;
            g.ecx += 1;
            if g.ecx == cars {
                break;
            }
        }
        g.ecx = 0;
        loop {
            let mine: u32 = game_read(ARCADE_ORDER_RVA + 4 * g.ecx as usize);
            part(18, &mut g)?;
            g.ebx = cars;
            let product = g.eax as u64 * cars as u64;
            g.eax = product as u32;
            g.edx = (product >> 32) as u32;
            let other: u32 = game_read(ARCADE_ORDER_RVA + 4 * g.edx as usize);
            game_write(ARCADE_ORDER_RVA + 4 * g.ecx as usize, other);
            game_write(ARCADE_ORDER_RVA + 4 * g.edx as usize, mine);
            g.ecx += 1;
            if g.ecx == cars {
                break;
            }
        }
        part(8, &mut g)?;
        for n in 0..4 {
            g.esi = game_addr(block(n)) as u32;
            part(17, &mut g)?;
        }
        game_write(block(0) + 0x3C, brake_value(0));
        for off in [0x66C, 0x65C, 0x660] {
            game_write(block(0) + off, 0u32);
        }
        g.eax = cars.wrapping_sub(game_read(START_POS_RVA));
        game_write(block(0) + 0x658, g.eax);
        log(9, w(6), &g, part)?;
        log(11, g.eax, &g, part)?;
        log(9, w(7), &g, part)?;
        g.ecx = 0;
        loop {
            log(
                11,
                game_read(ARCADE_KINDS_RVA + 4 * g.ecx as usize),
                &g,
                part,
            )?;
            g.ecx += 1;
            if g.ecx >= cars {
                break;
            }
        }
        log(13, g.eax, &g, part)?;
    }
    Ok(g)
}

/// What `set_up_mode_2_cars` and `set_up_cars` write beyond `set_up_regions`.
pub fn arcade_regions() -> Vec<Region> {
    let mut r = set_up_regions();
    r.extend(race_table_regions());
    r.extend(
        [
            ARCADE_CARS_RVA,
            ARCADE_A_RVA,
            ARCADE_B_RVA,
            START_POS_RVA,
            BEST_POS_RVA,
            NOISE_STATE_RVA,
            RALLY_RVA,
            STAGE_IN_RALLY_RVA,
            ARCADE_KIND_RVA,
            NOISE_SEED_RVA,
            KIND_FACTOR_RVA,
            EXTRA_TIME_RVA,
            CHECK_POSITS_RVA,
            STAGE_TOTAL_RVA,
            READ_BYTES_RVA,
        ]
        .map(at),
    );
    r.push((
        game_addr(ARCADE_KINDS_RVA) as u32,
        4 * MOST_ARCADE_CARS as usize,
    ));
    r.push((
        game_addr(ARCADE_ORDER_RVA) as u32,
        4 * MOST_ARCADE_CARS as usize,
    ));
    r.push((game_addr(SPEEDS_RVA) as u32, 4 * MOST_NUMBERS));
    r.push((game_addr(rva(0x601C68)) as u32, 0x100));
    r
}

/// `race::set_up_cars` (0x43603C): race init's "cars": "gamemode" and `dword_62389C`
/// logged; both players' +0x5FC 0 and +0xA0C .. +0xA1C -1; then by the mode: 2 the
/// numbers, the times, the arcade set-up, `dword_601C68` (0x40 dwords) 0; 1, 3, 0 (with
/// the numbers first), 4 their set-ups; others nothing.
pub unsafe fn set_up_cars(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        log(9, game_addr(rva(0x4DB9EB)) as u32, &g, part)?;
        log(12, game_read(MODE_RVA), &g, part)?;
        for n in 0..2 {
            game_write(block(n) + 0x5FC, 0u32);
            for k in [0xA0C, 0xA14, 0xA10, 0xA1C, 0xA18] {
                game_write(block(n) + k, u32::MAX);
            }
        }
        match game_read::<u32>(MODE_RVA) {
            2 => {
                part(1, &mut g)?;
                part(2, &mut g)?;
                part(3, &mut g)?;
                for k in 0..0x40 {
                    game_write(rva(0x601C68) + 4 * k, 0u32);
                }
                g.edi = game_addr(rva(0x601C68 + 0x100)) as u32;
                g.eax = 0;
                g.ecx = 0;
            }
            1 => part(4, &mut g)?,
            3 => part(5, &mut g)?,
            0 => {
                part(1, &mut g)?;
                part(6, &mut g)?;
            }
            4 => part(7, &mut g)?,
            _ => {}
        }
    }
    Ok(g)
}

/// The inputs of an arcade case: the rally, the stage in it, `dword_60101C`, the noise
/// seed, the two players' numbers (+0xAA8).
pub fn prepare_arcade(rally: u32, stage_in: u32, kind: u32, seed: u32, taken: [u32; 2]) {
    unsafe {
        game_write(RALLY_RVA, rally);
        game_write(STAGE_IN_RALLY_RVA, stage_in);
        game_write(ARCADE_KIND_RVA, kind);
        game_write(NOISE_SEED_RVA, seed);
        for (n, &k) in taken.iter().enumerate() {
            game_write(block(n) + 0xAA8, k);
        }
    }
}

// ---- the arcade's opponents ----

const SEED_WORD_RVA: usize = rva(0x4DD90E); // "seed"
/// "p1", "power", "p2", "power", "p3", "power", "p4", "power".
const OPPONENT_WORDS: [usize; 8] = [
    rva(0x4DD913),
    rva(0x4DD916),
    rva(0x4DD91C),
    rva(0x4DD91F),
    rva(0x4DD925),
    rva(0x4DD928),
    rva(0x4DD92E),
    rva(0x4DD931),
];
const SETUP_ARCADE_WORD_RVA: usize = rva(0x4DC68A); // "SETUPARCADECARS"

/// The kinds a draw (0 .. 0x21) can give that `choose_arcade_opponents` takes: usable
/// (`kind_unusable` clear) and within 5 places (`index_of_kind`) of the first car's.
unsafe fn opponent_kinds(first: u32) -> Vec<u32> {
    unsafe {
        let ebp = crate::race::race_shell::index_of_kind(&Regs {
            eax: first,
            ..Regs::default()
        })
        .eax;
        (0..0x22u32)
            .filter(|&k| {
                let place = crate::race::race_shell::index_of_kind(&Regs {
                    eax: k,
                    ..Regs::default()
                })
                .eax;
                !crate::race::race_shell::kind_unusable(k).0
                    && place.wrapping_sub(ebp).wrapping_abs_i32() < 5
                    && k != first
            })
            .collect()
    }
}

trait WrappingAbs {
    fn wrapping_abs_i32(self) -> u32;
}

impl WrappingAbs for u32 {
    /// `cmp eax, 0` / `jns` / `neg`.
    fn wrapping_abs_i32(self) -> u32 {
        if (self as i32) < 0 {
            self.wrapping_neg()
        } else {
            self
        }
    }
}

/// `race::choose_arcade_opponents` (0x449B79): with `dword_62389C` 2, the first car's
/// kind's place (`index_of_kind`) in ebp, logged ("seed", "p1", "power"); from the noise
/// seed (`dword_60519C`; then `math::next_random`) kinds drawn (x 0x22 >> 32) until one is
/// usable, within 5 places of the first's and not taken: the second, third and fourth
/// cars' (`dword_62385C`, `60`, `64`), each logged with its place. Leaves eax the last
/// place difference, ebx its place, ecx as `index_of_kind`, edx its kind, ebp the first's
/// place. Refuses, before anything, a first kind with fewer than three such kinds (the
/// original draws for ever).
pub unsafe fn choose_arcade_opponents(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    use crate::race::race_shell::{index_of_kind, kind_unusable};
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(MODE_RVA) != 2 {
            return Ok(g);
        }
        let first: u32 = game_read(CAR_KINDS_RVA);
        if opponent_kinds(first).len() < 3 {
            return Err(BadInput("fewer than three opponents can be drawn").into());
        }
        g.eax = first;
        g = index_of_kind(&g);
        g.ebp = g.eax;
        log(9, game_addr(SEED_WORD_RVA) as u32, &g, part)?;
        log(12, game_read(NOISE_SEED_RVA), &g, part)?;
        log(9, game_addr(OPPONENT_WORDS[0]) as u32, &g, part)?;
        log(11, first, &g, part)?;
        log(9, game_addr(OPPONENT_WORDS[1]) as u32, &g, part)?;
        log(12, g.ebp, &g, part)?;
        game_write(NOISE_STATE_RVA, game_read::<u32>(NOISE_SEED_RVA));
        let mut taken = vec![first];
        for (n, at) in [rva(0x62385C), rva(0x623860), rva(0x623864)]
            .into_iter()
            .enumerate()
        {
            loop {
                g.eax = if n == 0 {
                    crate::scene::next_noise::next_noise()
                } else {
                    crate::math::next_random::next_random()
                };
                g.ebx = 0x22;
                g.edx = ((g.eax as u64 * 0x22) >> 32) as u32;
                g.eax = g.edx;
                if kind_unusable(g.eax).0 {
                    continue;
                }
                g = index_of_kind(&g);
                g.ebx = g.eax;
                g.eax = g.eax.wrapping_sub(g.ebp).wrapping_abs_i32();
                if g.eax >= 5 || taken.contains(&g.edx) {
                    continue;
                }
                break;
            }
            taken.push(g.edx);
            game_write(at, g.edx);
            log(9, game_addr(OPPONENT_WORDS[2 * n + 2]) as u32, &g, part)?;
            log(11, g.edx, &g, part)?;
            log(9, game_addr(OPPONENT_WORDS[2 * n + 3]) as u32, &g, part)?;
            log(12, g.ebx, &g, part)?;
        }
    }
    Ok(g)
}

/// `race::set_up_arcade_cars` (0x4468FB): with `dword_62389C` 2, "SETUPARCADECARS"
/// logged, `choose_arcade_opponents` (the log indented round it), `g_raceCars` 4; then
/// each block's +0x24 its kind (`g_raceCarKinds` and the seven after). Leaves the
/// registers as `choose_arcade_opponents`.
pub unsafe fn set_up_arcade_cars(entry: &Regs, indent: u32, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(MODE_RVA) == 2 {
            log(10, game_addr(SETUP_ARCADE_WORD_RVA) as u32, &g, part)?;
            let at = indent as *mut u32;
            at.write_unaligned(at.read_unaligned().wrapping_add(3));
            part(19, &mut g)?;
            at.write_unaligned(at.read_unaligned().wrapping_sub(3));
            game_write(CARS_RVA, 4u32);
        }
        for n in 0..8 {
            game_write(block(n) + 0x24, game_read::<u32>(CAR_KINDS_RVA + 4 * n));
        }
    }
    Ok(g)
}

/// What the two write, with their inputs.
pub fn opponent_regions() -> Vec<Region> {
    let mut r = vec![
        (game_addr(CAR_KINDS_RVA) as u32, 32),
        at(NOISE_STATE_RVA),
        at(NOISE_SEED_RVA),
        at(crate::math::next_random::STATE_RVA),
        at(MODE_RVA),
        at(CARS_RVA),
    ];
    r.extend((0..8).map(|n| at(block(n) + 0x24)));
    r
}
