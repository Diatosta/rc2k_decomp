//! `menu::shuffle_drivers`: meant to shuffle four driver tables (three menu tables and the
//! drivers; 0x94 an entry; counts `state.menus.class_counts`), but it never adds the two random
//! entries' offsets: for each table, count times, two random numbers under the count
//! and the first entry swapped with itself through the first
//! shuffle table / the second shuffle table (both left holding it). A 1999 slip kept as is.
//!
//! Callers: one of the game's. The random numbers are `math::random_below`'s. Refuses a count
//! of 0 (it would divide by 0), before anything.
use crate::cells::Store;
use crate::errors::{BadInput, Refused};
use crate::menu::drivers_store::{DRIVER_LEN, records_from_bytes, records_to_bytes};
use crate::menu::owner::Menus;
use crate::session::PlainDword;

/// The four classes' first drivers (the drivers + 90, + 140, + 170, + 0) and counts.
fn tables(menus: &Menus) -> [(usize, &PlainDword); 4] {
    let counts = &menus.screens.state.menus.class_counts;
    [
        (90, &counts[1]),
        (140, &counts[2]),
        (170, &counts[3]),
        (0, &counts[0]),
    ]
}

pub fn shuffle_drivers(menus: &Menus) -> Result<(), Refused> {
    use crate::math::random_below::random_below;
    let tables = tables(menus);
    if tables.iter().any(|&(_, c)| c.get() == 0) {
        return Err(BadInput("a driver table count of 0").into());
    }
    for &(table, count) in &tables {
        let first = &menus.cars.drivers.drivers[table];
        let n = count.get();
        for _ in 0..n {
            // Two draws nothing uses: they advance the random generator, which later draws
            // (and so replays) depend on.
            random_below(n)?;
            random_below(n)?;
            // The first entry through both scratch records and back (both left holding it).
            let driver = records_to_bytes(std::slice::from_ref(first));
            debug_assert_eq!(driver.len(), DRIVER_LEN);
            menus.cars.numbers.shuffle_a.store(0, &driver)?;
            menus.cars.numbers.shuffle_b.store(0, &driver)?;
            records_from_bytes(std::slice::from_ref(first), &driver)?;
        }
    }
    Ok(())
}
