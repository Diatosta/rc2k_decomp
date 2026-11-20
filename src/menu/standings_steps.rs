//! The event drivers' steps on the two standings tables (A: the rally's, B: the
//! championship's; see `menu::standings`): clearing a table's figures and numbering its
//! entries. Each refuses a table without its end entry (or a player past 4 or missing)
//! before changing anything.

use crate::errors::Refused;
use crate::menu::owner::{Play, Screens};
use crate::menu::standings::*;
use rc2k_core::menu::standings::Numbering;

/// Gives every entry of `table` a figure of 0 (table A, entry +0x44) or a position of 0
/// (table B, +0x70).
pub(super) fn clear_standings(
    table: Which,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    match table {
        Which::A => {
            let n = walked_a(tables)?;
            with_a(tables, |t| {
                t.entries[..n].iter_mut().for_each(|e| e.head.figure = 0);
            })?;
        }
        Which::B => {
            let n = walked_b(tables)?;
            with_b(tables, |t| {
                t.entries[..n].iter_mut().for_each(|e| e.position = 0);
            })?;
        }
    }
    Ok(())
}

/// Numbers the entries of `table` 1, 2, .. in order (with the class mode 2 only those with
/// a group), each player's number also into its record. Table A's count goes into
/// `EventState.standings_count`.
pub(super) fn number_all(
    screens: &Screens,
    play: &Play,
    table: Which,
    tables: &crate::saves::best_times::BestTables,
) -> Result<Numbered, Refused> {
    let grouped = screens.state.menus.class_mode.get() == 2;
    let numbering = Numbering::All { grouped };
    let numbered = match table {
        Which::A => number_table(play, tables, table, numbering, |p| &mut p.number_all_a)?,
        Which::B => number_table(play, tables, table, numbering, |p| &mut p.number_all_b)?,
    };
    if table == Which::A {
        screens.event.standings_count.set(numbered.last_number);
    }
    Ok(numbered)
}

/// Numbers the entries of group `class` of `table` 1, 2, .. in order, each player's number
/// also into its record.
pub(super) fn number_standings(
    play: &Play,
    table: Which,
    class: u32,
    tables: &crate::saves::best_times::BestTables,
) -> Result<Numbered, Refused> {
    let numbering = Numbering::Group(class);
    match table {
        Which::A => number_table(play, tables, table, numbering, |p| {
            &mut p.number_standings_a
        }),
        Which::B => number_table(play, tables, table, numbering, |p| {
            &mut p.number_standings_b
        }),
    }
}
