//! `menu::shuffle_drivers`: shuffles the four driver tables (three menu tables and the
//! drivers; 0x94 an entry; counts `CLASS_DRIVER_COUNTS`): for each table, count times, two
//! entries picked at random under the count swap places (whole records).
//!
//! The random numbers are `Randoms::random_below`'s, two per swap. Refuses a count of 0 (it would
//! divide by 0), before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use rc2k_core::saves::defaults::CLASS_DRIVER_COUNTS;

/// The four classes' first drivers (the drivers + 90, + 140, + 170, + 0) and counts.
fn tables() -> [(usize, u32); 4] {
    let counts = CLASS_DRIVER_COUNTS;
    [
        (90, counts[1]),
        (140, counts[2]),
        (170, counts[3]),
        (0, counts[0]),
    ]
}

pub(super) fn shuffle_drivers(
    menus: &Menus,
    random: &crate::math::next_random::Randoms,
) -> Result<(), Refused> {
    let tables = tables();
    if tables.iter().any(|&(_, c)| c == 0) {
        return Err(BadInput("a driver table count of 0").into());
    }
    let drivers = &menus.cars.drivers;
    for &(table, count) in &tables {
        for _ in 0..count {
            let first = table + random.random_below(count)? as usize;
            let second = table + random.random_below(count)? as usize;
            drivers.swap(first, second);
        }
    }
    Ok(())
}
