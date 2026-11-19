//! The current player's inspection figures from the car's table (the car table's row for the
//! player's car; car 0x21 counts as 0x17).
//! - For each of the fourteen parts: its repair time (`repair_times`) from the wear and the
//!   table's pair of low / high ends, its high end (`highs`) and its limit (`limits`).
//! - The high ends' total but part 4's goes into `total`; some of the table's and player's
//!   figures are copied beside it.
//! - For each inspected part (`menu::standings::INSPECTED`): its weight (`weights`) is its high
//!   end / the total times `steps`, logged as `DriverStats.<part>`; when the wear is at or past
//!   the limit its penalty (`penalties`) is the wear / `steps` times the weight.
//!
//! Refuses a current player past 3 or missing, a car past 0xFF or without its table, before
//! anything.
use crate::errors::Fault;
use crate::errors::{BadInput, Refused};
use crate::memory::session::write_all;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::standings::*;

/// The parts' names in the log, by inspected part.
const NAMES: [&std::ffi::CStr; 13] = [
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
/// The text logged after each part (one copy per part, all the same).
const EQUALS: [&std::ffi::CStr; 13] = [
    c"eax=", c"eax=", c"eax=", c"eax=", c"eax=", c"eax=", c"eax=", c"eax=", c"eax=", c"eax=",
    c"eax=", c"eax=", c"eax=",
];
const PARTS: u32 = 14;
/// The current player and its car's table.
pub fn player_and_table<'a>(
    menus: &'a Menus,
    screens: &Screens,
) -> Result<(PlayerCell<'a>, crate::menu::cars_store::DescriptionTable), BadInput> {
    let n: u32 = screens.event.current_player.get();
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing"));
    }
    let p = player(menus, n);
    Ok((p, car_table(menus, p)?))
}

/// The player `p`'s car's table (the car table's row for the car, +0x74; car 0x21 as 0x17).
pub fn car_table(
    menus: &Menus,
    p: PlayerCell,
) -> Result<crate::menu::cars_store::DescriptionTable, BadInput> {
    let car = match p.get().car_number {
        0x21 => 0x17,
        c => c,
    };
    // Entries up to 0xFF are possible, past the table's 34: refused there.
    crate::menu::cars_store::description_table(
        &menus.cars.table,
        crate::menu::cars_store::car_entry(menus, car)?,
    )
}

pub fn driver_stats(menus: &Menus, screens: &Screens) -> Result<(), Refused> {
    use crate::logfile::log_int::log_int;
    use rc2k_core::menu::driver_figures::{Car, Constants, Table, figures};
    let (p, t) = player_and_table(menus, screens)?;
    let fault = Fault::default();
    let table = Table::read(|o| fault.ok(t.dword(&menus.cars.table, o)));
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
    write_all(&menus.service.repair_times, f.values);
    write_all(&screens.state.driver_stats.highs, f.highs);
    write_all(&screens.state.driver_stats.limits, f.limits);
    screens.state.driver_stats.total.set(f.total);
    let setups = &menus.service.setups;
    for (cell, value) in [
        &setups[2].chosen,
        &setups[2].installed,
        &setups[3].chosen,
        &setups[3].installed,
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
    for (slot, line) in f.logged.iter().enumerate() {
        crate::logfile::log_write::log_nul(&NAMES[slot], false, 1);
        log_int(line.wear as i32, false, 1);
        crate::logfile::log_write::log_nul(&EQUALS[slot], false, 1);
        log_int(line.limit as i32, true, 1);
    }
    Ok(())
}
