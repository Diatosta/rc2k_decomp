//! `menu::driver_stats` (0x4A73D8): the current player's (`dword_75598C`) inspection
//! figures from its car's table (`off_7564A0`[car]'s +0x74; car 0x21 as 0x17).
//! - For each of the fourteen parts k (wear the player's +0xD4 + 4k; the table's pair at
//!   +8 (k / 2), +0x38 for odd k): its value (`dword_75AB60`[k]) 0 without wear, the
//!   pair's low end when the pair is flat, else that plus (high - low) / `flt_75AB58` times
//!   (wear - 1) (`hhmm` the difference); its high end into `dword_75ABA0`[k]; its limit
//!   (`dword_75ABD8`[k]) the table's +0x78 + 4k, 0xA for part 1 unless the player's +0x278
//!   has all of 0x1E0, for part 13 unless it has all of 0x1E.
//! - The high ends' total but part 4's into `dword_75AC84`; `dword_75AB98` / `9C` the
//!   table's +0x70 / +0x74, `dword_75AB90` / `94` its +0x30 / +0x68; `dword_75ACA8` /
//!   `AC` the player's +0x114, `dword_75ACB4` / `B8` its +0x118.
//! - For each inspected part (`menu::standings::INSPECTED`): its weight
//!   (`dword_75AC10`[k]) its high end / the total times `flt_75AB5C`; logged
//!   ("DriverStats.<part>", the wear, "eax=", the limit: 0xA for part 1 unless
//!   `dword_75ACC4` is 1, for part 5 when `dword_75ACC8` is 1); when the wear is at or past
//!   that limit, its penalty (`dword_75AC48`[k]) the wear / `flt_75AB5C` times the weight.
//!   Each figure also passes through `dword_7559EC`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_48E093`, `sub_4A15E3`, `sub_4A8556`. Logs through the `logfile::` Rust
//! versions (level 1). Refuses a current player past 3 or missing, a car past 0xFF or
//! without its table, before anything. Shadow mode runs the original alone (it logs).
//! Self-test: the original and the Rust version on the game's player
//! and cars, wear and switches by lot, at the test precisions (exact at 0x027F; the whole
//! numbers made by the float steps within one at 0x037F); put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::{Floats, ROUNDED, at_test_precisions, check_state_floats};
use crate::service::harness::{Region, at, capture, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::driver_stats", rva(0x4A73D8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const CARS_RVA: usize = rva(0x7564A0); // off_7564A0
const VALUES_RVA: usize = rva(0x75AB60); // dword_75AB60: 14
const HIGHS_RVA: usize = rva(0x75ABA0); // dword_75ABA0: 14
const WEIGHTS_RVA: usize = rva(0x75AC10); // dword_75AC10: 14
const TOTAL_RVA: usize = rva(0x75AC84); // dword_75AC84
const STEPS_RVA: usize = rva(0x75AB58); // flt_75AB58
const SCALE_RVA: usize = rva(0x75AB5C); // flt_75AB5C
const FIGURE_RVA: usize = rva(0x7559EC); // dword_7559EC
const SPAN_RVA: usize = rva(0x7559F0); // hhmm
const KEPT_RVA: [usize; 4] = [rva(0x75ACA8), rva(0x75ACAC), rva(0x75ACB4), rva(0x75ACB8)];
const SWITCH_RVA: [usize; 2] = [rva(0x75ACC4), rva(0x75ACC8)]; // dword_75ACC4, dword_75ACC8
/// The parts' names in the log, by inspected part.
const NAMES_RVA: [usize; 13] = [
    rva(0x4DED08),
    rva(0x4DED20),
    rva(0x4DED3A),
    rva(0x4DED53),
    rva(0x4DED6E),
    rva(0x4DED86),
    rva(0x4DED9F),
    rva(0x4DEDB9),
    rva(0x4DEDD1),
    rva(0x4DEDED),
    rva(0x4DEE0B),
    rva(0x4DEE23),
    rva(0x4DEE3C),
];
/// The "eax=" after each (the game has one copy per part; all the same text).
const EQUALS_RVA: [usize; 13] = [
    rva(0x4DED1B),
    rva(0x4DED35),
    rva(0x4DED4E),
    rva(0x4DED69),
    rva(0x4DED81),
    rva(0x4DED9A),
    rva(0x4DEDB4),
    rva(0x4DEDCC),
    rva(0x4DEDE8),
    rva(0x4DEE06),
    rva(0x4DEE1E),
    rva(0x4DEE37),
    rva(0x4DEE52),
];
const PARTS: u32 = 14;
const EXTRA: [usize; 5] = [
    CURRENT_RVA,
    SWITCH_RVA[0],
    SWITCH_RVA[1],
    STEPS_RVA,
    SCALE_RVA,
];
const CASES: u32 = 4;

fn test_entry(_case: u32) -> Regs {
    Regs::SENTINEL
}

fn prepare(case: u32) {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n < 4 && player(n) != 0 {
        let p = player(n);
        for k in 0..PARTS {
            put(
                p + 0xD4 + 4 * k,
                [0u32, 1, 7, 30, 100][((k + case) % 5) as usize],
            );
        }
        put(p + 0x278, [0x1FE, 0, 0x1E0, 0x1E][case as usize]);
    }
    unsafe {
        game_write(SWITCH_RVA[0], u32::from(case % 2 == 1));
        game_write(SWITCH_RVA[1], u32::from(case >= 2));
    }
}

/// The current player and its car's table.
pub fn player_and_table() -> Result<(u32, u32), BadInput> {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing"));
    }
    let p = player(n);
    Ok((p, car_table(p)?))
}

/// The player `p`'s car's table (`off_7564A0`[car]'s +0x74; car 0x21 as 0x17).
pub fn car_table(p: u32) -> Result<u32, BadInput> {
    let car = match get(p + 8) {
        0x21 => 0x17,
        c => c,
    };
    let entry = if car < 0x100 {
        unsafe { game_read::<u32>(CARS_RVA + 4 * car as usize) }
    } else {
        0
    };
    if entry == 0 || get(entry + 0x74) == 0 {
        return Err(BadInput("a car past 0xFF or without its table"));
    }
    Ok(get(entry + 0x74))
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn driver_stats(entry: &Regs) -> Result<Regs, Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    // Converted: `rc2k_core::menu::driver_figures` (float: exact at the game's 0x027F).
    use rc2k_core::menu::driver_figures::{Car, Constants, Table, figures};
    let (p, t) = player_and_table()?;
    unsafe {
        let table = Table::read(|o| get(t + o as u32));
        let car = Car {
            wear: std::array::from_fn(|k| get(p + 0xD4 + 4 * k as u32)),
            fitted: get(p + 0x278),
            setup: [get(p + 0x114), get(p + 0x118)],
        };
        let c = Constants {
            steps: f32::from_bits(game_read(STEPS_RVA)),
            scale: f32::from_bits(game_read(SCALE_RVA)),
            keep_part_1: game_read(SWITCH_RVA[0]),
            short_part_5: game_read(SWITCH_RVA[1]),
        };
        let f = figures(&table, &car, &c);
        game_write(VALUES_RVA, f.values);
        game_write(HIGHS_RVA, f.highs);
        game_write(LIMITS_RVA, f.limits);
        game_write(TOTAL_RVA, f.total);
        for (n, &at) in KEPT_RVA.iter().enumerate() {
            game_write(at, f.kept[n]);
        }
        for k in 0..PARTS as usize {
            if let Some(w) = f.weights[k] {
                game_write(WEIGHTS_RVA + 4 * k, w);
            }
            if let Some(v) = f.penalties[k] {
                game_write(PENALTIES_RVA + 4 * k, v);
            }
        }
        game_write(FIGURE_RVA, f.figure);
        if let Some(span) = f.span {
            game_write(SPAN_RVA, span);
        }
        for (slot, line) in f.logged.iter().enumerate() {
            log_write(game_addr(NAMES_RVA[slot]) as *const u8, false, 1);
            log_int(line.wear as i32, false, 1);
            log_write(game_addr(EQUALS_RVA[slot]) as *const u8, false, 1);
            log_int(line.limit as i32, true, 1);
        }
    }
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    vec![
        (game_addr(VALUES_RVA) as u32, 0x75AC88 - 0x75AB60),
        (game_addr(KEPT_RVA[0]) as u32, 0x14),
        at(FIGURE_RVA),
        at(SPAN_RVA),
        (player_and_table().map(|(p, _)| p).unwrap_or(0), 0x280),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { driver_stats(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    // It logs: in shadow mode the original alone (the self-test compares the two).
    if crate::framework::shadow() {
        HOOK.count_call();
        unsafe { call_original(r) };
        return;
    }
    let regions = regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Unpaused: frames drawn meanwhile may see the tables this test changes.
    let _disturbing = crate::screens::harness::Disturbing::begin();
    let regions = regions();
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { driver_stats(&test_entry(0)) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    // The whole numbers made by the float steps: a rounding edge may differ by one at
    // extended precision.
    let places = [
        ((game_addr(VALUES_RVA) as u32, 14 * 4), ROUNDED),
        ((game_addr(WEIGHTS_RVA) as u32, 2 * 14 * 4), ROUNDED),
        (at(FIGURE_RVA), ROUNDED),
    ];
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for case in 0..CASES {
            check_state_floats(
                &mut t,
                "menu::driver_stats",
                &format!("control word {word:#x}, case {case}"),
                call_original,
                test_entry(case),
                &|| {
                    restore(&vars, &saved);
                    prepare(case);
                },
                &regions,
                held,
                &rust,
            );
        }
    });
    restore(&vars, &saved);
    t.finish()
}
