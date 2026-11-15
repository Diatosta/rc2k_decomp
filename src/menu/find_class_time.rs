//! `menu::find_class_time`: the first entry of the list (0x9C bytes each) whose class (+0x1C)
//! is the given one, or the end mark's (-1) when none is: its +0x18 when asked for that, else
//! its +0x14.
//!
//! Caller: `screens::draw_class_times` (the first time of the class). Refuses a list without
//! its end in 0x400 entries.

use crate::errors::BadInput;
use crate::menu::standings::with_a;
use rc2k_core::saves::entries::BestTime;

const LIMIT: u32 = 0x400;

/// The entry of table A the search stops at (a copy).
pub fn class_entry(
    class: u32,
    tables: &crate::saves::best_times::BestTables,
) -> Result<BestTime, BadInput> {
    use rc2k_core::menu::standings::{Found, search};
    with_a(tables, |t| {
        let mut last = 0;
        let found = search(
            |n| {
                last = n;
                t.first_at(n as usize).unwrap_or(u32::MAX)
            },
            |n| {
                t.entries
                    .get(n as usize)
                    .is_some_and(|e| e.head.group == class)
            },
            LIMIT,
        );
        match found {
            Found::At(n) => Ok(t.entries[n as usize]),
            Found::End => t
                .entries
                .get(last as usize)
                .copied()
                .ok_or(BadInput("table A's end mark past its entries")),
            Found::NoEnd => Err(BadInput("a list without its end in 0x400 entries")),
        }
    })?
}

/// Table A's.
pub fn find_class_time(
    class: u32,
    second: bool,
    tables: &crate::saves::best_times::BestTables,
) -> Result<u32, BadInput> {
    let e = class_entry(class, tables)?;
    Ok(if second {
        e.head.rally_time
    } else {
        e.head.key
    })
}
