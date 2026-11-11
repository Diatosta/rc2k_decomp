//! The current player's inspection figures from the car's table (the car table's row for the
//! player's car, +0x74; car 0x21 counts as 0x17).
//! - For each of the fourteen parts k (wear the player's +0xD4 + 4k; the table's pair at
//!   +8 (k / 2), +0x38 for odd k): its repair time (`driver_stats.repair_times`[k]) is 0
//!   without wear, the pair's low end when the pair is flat, else that plus (high - low) /
//!   `scale` times (wear - 1); its high end goes into `highs`[k]; its limit (`limits`[k]) is
//!   the table's +0x78 + 4k, 0xA for part 1 unless the player's +0x278 has all of 0x1E0, for
//!   part 13 unless it has all of 0x1E.
//! - The high ends' total but part 4's goes into `total`; the table's +0x70 / +0x74 and
//!   +0x30 / +0x68 and the player's +0x114 / +0x118 are copied to the figures beside it.
//! - For each inspected part (`menu::standings::INSPECTED`): its weight (`weights`[k]) is its
//!   high end / the total times `steps`; logged ("DriverStats.<part>", the wear, "eax=", the
//!   limit: 0xA for part 1 unless `switch_a` is 1, for part 5 when `switch_b` is 1); when
//!   the wear is at or past that limit, its penalty (`penalties`[k]) is the wear / `steps`
//!   times the weight. Each figure also passes through the draw work's figure.
//!
//! Logs through `logfile::`
//! (level 1). Refuses a current player past 3 or missing, a car past 0xFF or without its table,
//! before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::Fault;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::standings::*;
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
pub fn player_and_table<'a>(
    menus: &'a Menus,
    screens: &Screens,
) -> Result<(PlayerCell<'a>, crate::pointers::GamePtr), BadInput> {
    let n: u32 = screens.event.current_player.get();
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing"));
    }
    let p = player(menus, n);
    Ok((p, car_table(menus, p)?))
}

/// The player `p`'s car's table (the car table's row for the car, +0x74; car 0x21 as 0x17).
pub fn car_table(menus: &Menus, p: PlayerCell) -> Result<crate::pointers::GamePtr, BadInput> {
    let car = match p.get().car_number {
        0x21 => 0x17,
        c => c,
    };
    // The original reads entries up to 0xFF, on past the table's 34: refused there.
    crate::menu::cars_store::description_table(crate::menu::cars_store::car_entry(menus, car)?)
}

pub fn driver_stats(menus: &Menus, screens: &Screens) -> Result<(), Refused> {
    use crate::logfile::log_int::log_int;
    // The logic is in `rc2k_core::menu::driver_figures` (float: exact at the game's 0x027F).
    use rc2k_core::menu::driver_figures::{Car, Constants, Table, figures};
    let (p, t) = player_and_table(menus, screens)?;
    let fault = Fault::default();
    let table = Table::read(|o| fault.read_at::<u32>(t.offset(o as u32), "a car's figures"));
    let car = Car {
        wear: std::array::from_fn(|k| p.get().damage[k]),
        fitted: p.get().state_flags,
        setup: [p.get().ride_height, p.get().suspension_stiffness],
    };
    fault.check()?;
    let c = Constants {
        steps: screens.state.driver_stats.steps.get(),
        scale: screens.state.driver_stats.scale.get(),
        keep_part_1: screens.state.driver_stats.switch_a.get(),
        short_part_5: screens.state.driver_stats.switch_b.get(),
    };
    let f = figures(&table, &car, &c);
    write_all(&screens.state.driver_stats.repair_times, f.values);
    write_all(&screens.state.driver_stats.highs, f.highs);
    write_all(&screens.state.driver_stats.limits, f.limits);
    screens.state.driver_stats.total.set(f.total);
    let ds = &screens.state.driver_stats;
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
            screens.state.driver_stats.weights[k].set(w);
        }
        if let Some(v) = f.penalties[k] {
            screens.state.driver_stats.penalties[k].set(v);
        }
    }
    screens.draw_work.figure.set(f.figure);
    if let Some(span) = f.span {
        screens.draw_work.hhmm.set(span);
    }
    for (slot, line) in f.logged.iter().enumerate() {
        crate::logfile::log_write::log_nul(&NAMES_RVA[slot], false, 1);
        log_int(line.wear as i32, false, 1);
        crate::logfile::log_write::log_nul(&EQUALS_RVA[slot], false, 1);
        log_int(line.limit as i32, true, 1);
    }
    Ok(())
}
