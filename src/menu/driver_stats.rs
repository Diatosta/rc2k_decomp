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
//! Callers: `sub_48E093`, `sub_4A15E3`, `sub_4A8556`. Logs through the `logfile::` Rust versions
//! (level 1). Refuses a current player past 3 or missing, a car past 0xFF or without its table,
//! before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_flow::Fault;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::standings::*;
use crate::menu::state_store::MENU_STATE;
use crate::session::write_all;

/// The parts' names in the log, by inspected part.
const NAMES_RVA: [&std::ffi::CStr; 13] = [
    c"DriverStats.Engine",
    c"DriverStats.Bodywork",
    c"DriverStats.Exhaust",
    c"DriverStats.Electrics",
    c"DriverStats.Lights",
    c"DriverStats.Cooling",
    c"DriverStats.Steering",
    c"DriverStats.Clutch",
    c"DriverStats.Suspension",
    c"DriverStats.Differential",
    c"DriverStats.Brakes",
    c"DriverStats.Gearbox",
    c"DriverStats.TyresWear",
];
/// The "eax=" after each (the game has one copy per part; all the same text).
const EQUALS_RVA: [&std::ffi::CStr; 13] = [
    c"eax=", c"eax=", c"eax=", c"eax=", c"eax=", c"eax=", c"eax=", c"eax=", c"eax=", c"eax=",
    c"eax=", c"eax=", c"eax=",
];
const PARTS: u32 = 14;
/// The current player and its car's table.
pub fn player_and_table() -> Result<(u32, u32), BadInput> {
    let n: u32 = EVENT_STATE.current_player.get();
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing"));
    }
    let p = player(n);
    Ok((p, car_table(p)?))
}

/// The player `p`'s car's table (`off_7564A0`[car]'s +0x74; car 0x21 as 0x17).
pub fn car_table(p: u32) -> Result<u32, BadInput> {
    let car = match get(p + 8)? {
        0x21 => 0x17,
        c => c,
    };
    // The original reads entries up to 0xFF, on past the table's 34: refused there.
    crate::menu::cars_store::description_table(crate::menu::cars_store::car_entry(car)?)
}

/// The Rust version (see the header): the registers it leaves.
pub fn driver_stats() -> Result<(), Refused> {
    use crate::logfile::log_int::log_int;
    // Converted: `rc2k_core::menu::driver_figures` (float: exact at the game's 0x027F).
    use rc2k_core::menu::driver_figures::{Car, Constants, Table, figures};
    let (p, t) = player_and_table()?;
    let fault = Fault::default();
    let table = Table::read(|o| fault.get(t + o as u32));
    let car = Car {
        wear: std::array::from_fn(|k| fault.get(p + 0xD4 + 4 * k as u32)),
        fitted: fault.get(p + 0x278),
        setup: [fault.get(p + 0x114), fault.get(p + 0x118)],
    };
    fault.check()?;
    let c = Constants {
        steps: MENU_STATE.driver_stats.steps.get(),
        scale: MENU_STATE.driver_stats.scale.get(),
        keep_part_1: MENU_STATE.driver_stats.switch_a.get(),
        short_part_5: MENU_STATE.driver_stats.switch_b.get(),
    };
    let f = figures(&table, &car, &c);
    write_all(&MENU_STATE.driver_stats.repair_times, f.values);
    write_all(&MENU_STATE.driver_stats.highs, f.highs);
    write_all(&MENU_STATE.driver_stats.limits, f.limits);
    MENU_STATE.driver_stats.total.set(f.total);
    let ds = &MENU_STATE.driver_stats;
    for (cell, value) in [
        &ds.setups[2].chosen,
        &ds.setups[2].installed,
        &ds.setups[3].chosen,
        &ds.setups[3].installed,
    ]
    .into_iter()
    .zip(f.kept)
    {
        cell.set(value);
    }
    for k in 0..PARTS as usize {
        if let Some(w) = f.weights[k] {
            MENU_STATE.driver_stats.weights[k].set(w);
        }
        if let Some(v) = f.penalties[k] {
            MENU_STATE.driver_stats.penalties[k].set(v);
        }
    }
    DRAW_WORK.figure.set(f.figure);
    if let Some(span) = f.span {
        DRAW_WORK.hhmm.set(span);
    }
    for (slot, line) in f.logged.iter().enumerate() {
        crate::logfile::log_write::log_nul(&NAMES_RVA[slot], false, 1);
        log_int(line.wear as i32, false, 1);
        crate::logfile::log_write::log_nul(&EQUALS_RVA[slot], false, 1);
        log_int(line.limit as i32, true, 1);
    }
    Ok(())
}
