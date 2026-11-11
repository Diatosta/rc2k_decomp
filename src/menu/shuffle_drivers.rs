//! `menu::shuffle_drivers` (0x470AB9): meant to shuffle four driver tables
//! (a menu table, a menu table, a menu table, `g_drivers`; 0x94 an entry; counts
//! `state.menus.class_counts`, `state.menus.class_counts`, `state.menus.class_counts`, `state.menus.class_counts`), but it never adds the
//! two random entries' offsets: for each table, count times, two random numbers under the
//! count (`draw_work.figure`, `hhmm`) and the first entry swapped with itself through
//! the first shuffle table / the second shuffle table (both left holding it). Section 16.
//!
//! Callers: one of the game's. The random numbers are `math::random_below`'s.
//! Refuses a count of 0 (the original's `div` traps, its loop runs on), before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::pointers::Store;
use crate::session::PlainDword;

const DRIVER_LEN: usize = 0x94;

/// The four classes' first drivers (`g_drivers` + 90, + 140, + 170, + 0) and counts.
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
            menus.screens.draw_work.figure.set(random_below(n)?);
            menus.screens.draw_work.hhmm.set(random_below(n)?);
            // The first entry through both scratch records and back (both left holding it).
            let mut driver = [0; DRIVER_LEN];
            first.load(0, &mut driver)?;
            menus.cars.numbers.shuffle_a.store(0, &driver)?;
            menus.cars.numbers.shuffle_b.store(0, &driver)?;
            first.store(0, &driver)?;
        }
    }
    Ok(())
}
