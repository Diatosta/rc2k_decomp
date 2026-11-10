//! `menu::shuffle_drivers` (0x470AB9): meant to shuffle four driver tables
//! (`unk_73EA7C`, `unk_740764`, `unk_7418BC`, `g_drivers`; 0x94 an entry; counts
//! `dword_75AB04`, `dword_75AB08`, `dword_75AB0C`, `dword_75AB00`), but it never adds the
//! two random entries' offsets: for each table, count times, two random numbers under the
//! count (`dword_7559EC`, `hhmm`) and the first entry swapped with itself through
//! `unk_755398` / `unk_75542C` (both left holding it). Section 16.
//!
//! Callers: one of the game's. The random numbers are `math::random_below`'s.
//! Refuses a count of 0 (the original's `div` traps, its loop runs on), before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::pointers::Store;
use crate::session::Dword;

const DRIVER_LEN: usize = 0x94;

/// The four classes' first drivers (`g_drivers` + 90, + 140, + 170, + 0) and counts.
const TABLES: [(usize, &Dword); 4] = [
    (
        90,
        &crate::menu::owner::MENUS.screens.state.menus.class_counts[1],
    ),
    (
        140,
        &crate::menu::owner::MENUS.screens.state.menus.class_counts[2],
    ),
    (
        170,
        &crate::menu::owner::MENUS.screens.state.menus.class_counts[3],
    ),
    (
        0,
        &crate::menu::owner::MENUS.screens.state.menus.class_counts[0],
    ),
];

pub fn shuffle_drivers(menus: &Menus) -> Result<(), Refused> {
    use crate::math::random_below::random_below;
    if TABLES.iter().any(|&(_, c)| c.get() == 0) {
        return Err(BadInput("a driver table count of 0").into());
    }
    for &(table, count) in &TABLES {
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
